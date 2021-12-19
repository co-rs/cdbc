use cdbc::error::Error;
use crate::statement::StatementHandle;

use either::Either;
use std::sync::{Arc, Weak};
use std::thread;

use crate::connection::ConnectionHandleRef;

use libsqlite3_sys::{sqlite3_reset, sqlite3_step, SQLITE_DONE, SQLITE_ROW};
use crossbeam_channel::Sender;


// Each SQLite connection has a dedicated thread.

// TODO: Tweak this so that we can use a thread pool per pool of SQLite3 connections to reduce
//       OS resource usage. Low priority because a high concurrent load for SQLite3 is very
//       unlikely.

pub(crate) struct StatementWorker {
    tx: Sender<StatementWorkerCommand>,
}

enum StatementWorkerCommand {
    Step {
        statement: Weak<StatementHandle>,
        tx: Sender<Result<Either<u64, ()>, Error>>,
    },
    Reset {
        statement: Weak<StatementHandle>,
        tx: Sender<()>,
    },
    Shutdown {
        tx: Sender<()>,
    },
}

impl StatementWorker {
    pub(crate) fn new(conn: ConnectionHandleRef) -> Self {
        let (tx, rx) = crossbeam_channel::unbounded();

        thread::spawn(move || {
            for cmd in rx {
                match cmd {
                    StatementWorkerCommand::Step { statement, tx } => {
                        let statement = if let Some(statement) = statement.upgrade() {
                            statement
                        } else {
                            // statement is already finalized, the sender shouldn't be expecting a response
                            continue;
                        };

                        // SAFETY: only the `StatementWorker` calls this function
                        let status = unsafe { sqlite3_step(statement.as_ptr()) };
                        let result = match status {
                            SQLITE_ROW => Ok(Either::Right(())),
                            SQLITE_DONE => Ok(Either::Left(statement.changes())),
                            _ => Err(statement.last_error().into()),
                        };

                        let _ = tx.send(result);
                    }
                    StatementWorkerCommand::Reset { statement, tx } => {
                        if let Some(statement) = statement.upgrade() {
                            // SAFETY: this must be the only place we call `sqlite3_reset`
                            unsafe { sqlite3_reset(statement.as_ptr()) };

                            // `sqlite3_reset()` always returns either `SQLITE_OK`
                            // or the last error code for the statement,
                            // which should have already been handled;
                            // so it's assumed the return value is safe to ignore.
                            //
                            // https://www.sqlite.org/c3ref/reset.html

                            let _ = tx.send(());
                        }
                    }
                    StatementWorkerCommand::Shutdown { tx } => {
                        // drop the connection reference before sending confirmation
                        // and ending the command loop
                        drop(conn);
                        let _ = tx.send(());
                        return;
                    }
                }
            }

            // SAFETY: we need to make sure a strong ref to `conn` always outlives anything in `rx`
            drop(conn);
        });

        Self { tx }
    }

    pub(crate) fn step(
        &mut self,
        statement: &Arc<StatementHandle>,
    ) -> Result<Either<u64, ()>, Error> {
        let (tx, rx) = crossbeam_channel::unbounded();

        self.tx
            .send(StatementWorkerCommand::Step {
                statement: Arc::downgrade(statement),
                tx,
            })
            .map_err(|e| {
                Error::WorkerCrashed(e.to_string())
            })?;

        rx.recv().map_err(|e| {
            Error::WorkerCrashed(e.to_string())
        })?
    }

    /// Send a command to the worker to execute `sqlite3_reset()` next.
    ///
    /// This method is written to execute the sending of the command eagerly so
    /// you do not need to await the returned future unless you want to.
    ///
    /// The only error is `WorkerCrashed` as `sqlite3_reset()` returns the last error
    /// in the statement execution which should have already been handled from `step()`.
    pub(crate) fn reset(
        &mut self,
        statement: &Arc<StatementHandle>,
    ) ->  Result<(), Error> {
        // execute the sending eagerly so we don't need to spawn the future
        let (tx, rx) = crossbeam_channel::unbounded();

        let send_res = self
            .tx
            .send(StatementWorkerCommand::Reset {
                statement: Arc::downgrade(statement),
                tx,
            })
            .map_err(|e| {
                Error::WorkerCrashed(e.to_string())
            });

         {
            send_res?;

            // wait for the response
            rx.recv().map_err(|e| {
                Error::WorkerCrashed(e.to_string())
            })
        }
    }

    /// Send a command to the worker to shut down the processing thread.
    ///
    /// A `WorkerCrashed` error may be returned if the thread has already stopped.
    /// Subsequent calls to `step()`, `reset()`, or this method will fail with
    /// `WorkerCrashed`. Ensure that any associated statements are dropped first.
    pub(crate) fn shutdown(&mut self) ->  Result<(), Error> {
        let (tx, rx) = crossbeam_channel::unbounded();

        let send_res = self
            .tx
            .send(StatementWorkerCommand::Shutdown { tx })
            .map_err(|e| {
                Error::WorkerCrashed(e.to_string())
            });

        {
            send_res?;

            // wait for the response
            rx.recv().map_err(|e| {
                Error::WorkerCrashed(e.to_string())
            })
        }
    }
}

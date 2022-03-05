use std::borrow::Cow;
use std::future::Future;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use mco::chan;
use mco::std::sync::channel::Sender;
use mco::std::sync::{channel, Mutex, MutexGuard};

use either::Either;
use cdbc::describe::Describe;
use cdbc::error::Error;
use crate::connection::collation::create_collation;
use crate::connection::describe::describe;
use crate::connection::establish::EstablishParams;
use crate::connection::ConnectionState;
use crate::connection::{execute};
use crate::{Sqlite, SqliteArguments, SqliteQueryResult, SqliteRow, SqliteStatement};
use crate::connection::handle::ConnectionHandleRaw;
use cdbc::transaction::{
    begin_ansi_transaction_sql, commit_ansi_transaction_sql, rollback_ansi_transaction_sql,
};

// Each SQLite connection has a dedicated thread.

// TODO: Tweak this so that we can use a thread pool per pool of SQLite3 connections to reduce
//       OS resource usage. Low priority because a high concurrent load for SQLite3 is very
//       unlikely.

pub struct ConnectionWorker {
    command_tx: Sender<Command>,
    /// The `sqlite3` pointer. NOTE: access is unsynchronized!
    pub handle_raw: ConnectionHandleRaw,
    /// Mutex for locking access to the database.
    pub shared: Arc<WorkerSharedState>,
}

pub struct WorkerSharedState {
    pub cached_statements_size: AtomicUsize,
    pub conn: Mutex<ConnectionState>,
}

enum Command {
    Prepare {
        query: Box<str>,
        tx: Sender<Result<SqliteStatement, Error>>,
    },
    Describe {
        query: Box<str>,
        tx: Sender<Result<Describe<Sqlite>, Error>>,
    },
    Execute {
        query: Box<str>,
        arguments: Option<SqliteArguments<'static>>,
        persistent: bool,
        tx: Sender<Result<Either<SqliteQueryResult, SqliteRow>, Error>>,
    },
    Begin {
        tx: Sender<Result<(), Error>>,
    },
    Commit {
        tx: Sender<Result<(), Error>>,
    },
    Rollback {
        tx: Option<Sender<Result<(), Error>>>,
    },
    CreateCollation {
        create_collation:
        Box<dyn FnOnce(&mut ConnectionState) -> Result<(), Error> + Send + Sync + 'static>,
    },
    UnlockDb,
    ClearCache {
        tx: Sender<()>,
    },
    Ping {
        tx: Sender<()>,
    },
    Shutdown {
        tx: Sender<()>,
    },
}

impl ConnectionWorker {
    pub fn establish(params: EstablishParams) -> Result<Self, Error> {
        let (establish_tx, establish_rx) = chan!();

        thread::Builder::new()
            .name(params.thread_name.clone())
            .spawn(move || {
                let (command_tx, command_rx) = chan!(params.command_channel_size);

                let conn = match params.establish() {
                    Ok(conn) => conn,
                    Err(e) => {
                        establish_tx.send(Err(e)).ok();
                        return;
                    }
                };

                let shared = Arc::new(WorkerSharedState {
                    cached_statements_size: AtomicUsize::new(0),
                    // note: must be fair because in `Command::UnlockDb` we unlock the mutex
                    // and then immediately try to relock it; an unfair mutex would immediately
                    // grant us the lock even if another task is waiting.
                    // conn: Mutex::new(conn, true),
                    conn: Mutex::new(conn),
                });
                let mut conn = shared.conn.try_lock().unwrap();

                if establish_tx
                    .send(Ok(Self {
                        command_tx,
                        handle_raw: conn.handle.to_raw(),
                        shared: Arc::clone(&shared),
                    }))
                    .is_err()
                {
                    return;
                }

                for cmd in command_rx {
                    match cmd {
                        Command::Prepare { query, tx } => {
                            tx.send(prepare(&mut conn, &query).map(|prepared| {
                                update_cached_statements_size(
                                    &conn,
                                    &shared.cached_statements_size,
                                );
                                prepared
                            }))
                                .ok();
                        }
                        Command::Describe { query, tx } => {
                            tx.send(describe(&mut conn, &query)).ok();
                        }
                        Command::Execute {
                            query,
                            arguments,
                            persistent,
                            tx,
                        } => {
                            let iter = match execute::iter(&mut conn, &query, arguments, persistent)
                            {
                                Ok(iter) => iter,
                                Err(e) => {
                                    tx.send(Err(e)).ok();
                                    continue;
                                }
                            };

                            for res in iter {
                                if tx.send(res).is_err() {
                                    break;
                                }
                            }

                            update_cached_statements_size(&conn, &shared.cached_statements_size);
                        }
                        Command::Begin { tx } => {
                            let depth = conn.transaction_depth;
                            let res =
                                conn.handle
                                    .exec(begin_ansi_transaction_sql(depth))
                                    .map(|_| {
                                        conn.transaction_depth += 1;
                                    });

                            tx.send(res).ok();
                        }
                        Command::Commit { tx } => {
                            let depth = conn.transaction_depth;

                            let res = if depth > 0 {
                                conn.handle
                                    .exec(commit_ansi_transaction_sql(depth))
                                    .map(|_| {
                                        conn.transaction_depth -= 1;
                                    })
                            } else {
                                Ok(())
                            };

                            tx.send(res).ok();
                        }
                        Command::Rollback { tx } => {
                            let depth = conn.transaction_depth;

                            let res = if depth > 0 {
                                conn.handle
                                    .exec(rollback_ansi_transaction_sql(depth))
                                    .map(|_| {
                                        conn.transaction_depth -= 1;
                                    })
                            } else {
                                Ok(())
                            };

                            if let Some(tx) = tx {
                                tx.send(res).ok();
                            }
                        }
                        Command::CreateCollation { create_collation } => {
                            if let Err(e) = (create_collation)(&mut conn) {
                                log::warn!("error applying collation in background worker: {:?}", e);
                            }
                        }
                        Command::ClearCache { tx } => {
                            conn.statements.clear();
                            update_cached_statements_size(&conn, &shared.cached_statements_size);
                            tx.send(()).ok();
                        }
                        Command::UnlockDb => {
                            drop(conn);
                            conn = {
                                 loop{
                                     if let Ok(v)=shared.conn.lock(){
                                         break v;
                                     }
                                 }
                            };
                        }
                        Command::Ping { tx } => {
                            tx.send(()).ok();
                        }
                        Command::Shutdown { tx } => {
                            // drop the connection reference before sending confirmation
                            // and ending the command loop
                            drop(conn);
                            let _ = tx.send(());
                            return;
                        }
                    }
                }
            })?;

        establish_rx.recv().map_err(|e| {
            Error::WorkerCrashed(e.to_string())
        })?
    }

    pub fn prepare(&mut self, query: &str) -> Result<SqliteStatement, Error> {
        self.oneshot_cmd(|tx| Command::Prepare {
            query: query.into(),
            tx,
        })?
    }

    pub fn describe(&mut self, query: &str) -> Result<Describe<Sqlite>, Error> {
        self.oneshot_cmd(|tx| Command::Describe {
            query: query.into(),
            tx,
        })?
    }

    pub fn execute(
        &mut self,
        query: &str,
        args: Option<SqliteArguments<'_>>,
        chan_size: usize,
        persistent: bool,
    ) -> Result<channel::Receiver<Result<Either<SqliteQueryResult, SqliteRow>, Error>>, Error> {
        let (tx, rx) = chan!(chan_size);
        self.command_tx
            .send(Command::Execute {
                query: query.into(),
                arguments: args.map(SqliteArguments::into_static),
                persistent,
                tx,
            })
            .map_err(|e| Error::WorkerCrashed(e.to_string()))?;
        Ok(rx)
    }

    pub fn begin(&mut self) -> Result<(), Error> {
        self.oneshot_cmd(|tx| Command::Begin { tx })?
    }

    pub fn commit(&mut self) -> Result<(), Error> {
        self.oneshot_cmd(|tx| Command::Commit { tx })?
    }

    pub fn rollback(&mut self) -> Result<(), Error> {
        self.oneshot_cmd(|tx| Command::Rollback { tx: Some(tx) })
            ?
    }

    pub fn start_rollback(&mut self) -> Result<(), Error> {
        self.command_tx
            .send(Command::Rollback { tx: None })
            .map_err(|e| Error::WorkerCrashed(e.to_string()))
    }

    pub fn ping(&mut self) -> Result<(), Error> {
        self.oneshot_cmd(|tx| Command::Ping { tx })
    }

    fn oneshot_cmd<F, T>(&mut self, command: F) -> Result<T, Error>
        where
            F: FnOnce(Sender<T>) -> Command,
    {
        let (tx, rx) = chan!();

        self.command_tx
            .send(command(tx))
            .map_err(|e| Error::WorkerCrashed(e.to_string()))?;

        rx.recv().map_err(|e| {
            Error::WorkerCrashed(e.to_string())
        })
    }

    pub fn create_collation(
        &mut self,
        name: &str,
        compare: impl Fn(&str, &str) -> std::cmp::Ordering + Send + Sync + 'static,
    ) -> Result<(), Error> {
        let name = name.to_string();

        self.command_tx
            .send(Command::CreateCollation {
                create_collation: Box::new(move |conn| {
                    create_collation(&mut conn.handle, &name, compare)
                }),
            })
            .map_err(|e| Error::WorkerCrashed(e.to_string()))?;
        Ok(())
    }

    pub fn clear_cache(&mut self) -> Result<(), Error> {
        self.oneshot_cmd(|tx| Command::ClearCache { tx })
    }

    pub fn unlock_db(&mut self) -> Result<MutexGuard<'_, ConnectionState>, Error> {
        // we need to join the wait queue for the lock before we send the message
        let guard = self.shared.conn.lock()?;
        let res = self.command_tx.send(Command::UnlockDb).map_err(|e| Error::WorkerCrashed(e.to_string()))?;
        Ok(guard)
    }

    /// Send a command to the worker to shut down the processing thread.
    ///
    /// A `WorkerCrashed` error may be returned if the thread has already stopped.
    pub fn shutdown(&mut self) -> Result<(), Error> {
        let (tx, rx) = chan!();

        let send_res = self
            .command_tx
            .send(Command::Shutdown { tx })
            .map_err(|e| Error::WorkerCrashed(e.to_string()))?;

        // wait for the response
        rx.recv().map_err(|e| {
            Error::WorkerCrashed(e.to_string())
        })
    }
}

fn prepare(conn: &mut ConnectionState, query: &str) -> Result<SqliteStatement, Error> {
    // prepare statement object (or checkout from cache)
    let statement = conn.statements.get(query, true)?;

    let mut parameters = 0;
    let mut columns = None;
    let mut column_names = None;

    while let Some(statement) = statement.prepare_next(&mut conn.handle)? {
        parameters += statement.handle.bind_parameter_count();

        // the first non-empty statement is chosen as the statement we pull columns from
        if !statement.columns.is_empty() && columns.is_none() {
            columns = Some(Arc::clone(statement.columns));
            column_names = Some(Arc::clone(statement.column_names));
        }
    }

    Ok(SqliteStatement {
        sql: query.to_string(),
        columns: columns.unwrap_or_default(),
        column_names: column_names.unwrap_or_default(),
        parameters,
    })
}

fn update_cached_statements_size(conn: &ConnectionState, size: &AtomicUsize) {
    size.store(conn.statements.len(), Ordering::Release);
}


use cdbc::describe::Describe;
use cdbc::error::Error;
use cdbc::executor::{Execute, Executor};
use crate::connection::describe::describe;
use crate::statement::{StatementHandle, StatementWorker, VirtualStatement};
use crate::{
    Sqlite, SqliteArguments, SqliteConnection, SqliteQueryResult, SqliteRow, SqliteStatement,
    SqliteTypeInfo,
};
use either::Either;
use libsqlite3_sys::sqlite3_last_insert_rowid;
use std::borrow::Cow;
use std::sync::Arc;
use cdbc::io::chan_stream::ChanStream;
use cdbc::utils::statement_cache::StatementCache;

fn prepare<'a>(
    worker: &mut StatementWorker,
    statements: &'a mut StatementCache<VirtualStatement>,
    statement: &'a mut Option<VirtualStatement>,
    query: &str,
    persistent: bool,
) -> Result<&'a mut VirtualStatement, Error> {
    if !persistent || !statements.is_enabled() {
        *statement = Some(VirtualStatement::new(query, false)?);
        return Ok(statement.as_mut().unwrap());
    }

    let exists = statements.contains_key(query);

    if !exists {
        let statement = VirtualStatement::new(query, true)?;
        statements.insert(query, statement);
    }

    let statement = statements.get_mut(query).unwrap();

    if exists {
        // as this statement has been executed before, we reset before continuing
        // this also causes any rows that are from the statement to be inflated
        statement.reset(worker)?;
    }

    Ok(statement)
}

fn bind(
    statement: &StatementHandle,
    arguments: &Option<SqliteArguments<'_>>,
    offset: usize,
) -> Result<usize, Error> {
    let mut n = 0;

    if let Some(arguments) = arguments {
        n += arguments.bind(statement, offset)?;
    }

    Ok(n)
}

/// A structure holding sqlite statement handle and resetting the
/// statement when it is dropped.
struct StatementResetter<'a> {
    handle: Arc<StatementHandle>,
    worker: &'a mut StatementWorker,
}

impl<'a> StatementResetter<'a> {
    fn new(worker: &'a mut StatementWorker, handle: &Arc<StatementHandle>) -> Self {
        Self {
            worker,
            handle: Arc::clone(handle),
        }
    }
}

impl Drop for StatementResetter<'_> {
    fn drop(&mut self) {
        // this method is designed to eagerly send the reset command
        // so we don't need to await or spawn it
        let _ = self.worker.reset(&self.handle);
    }
}

impl<'c> Executor for &'c mut SqliteConnection {
    type Database = Sqlite;

    fn fetch_many<'q, E: 'q>(
        self,
        mut query: E,
    ) -> ChanStream<Either<SqliteQueryResult, SqliteRow>>
    where
        E: Execute<'q, Self::Database>,
    {
        let sql = query.sql();
        let arguments = query.take_arguments();
        let persistent = query.persistent() && arguments.is_some();
        chan_stream! ({
            let SqliteConnection {
                handle: ref mut conn,
                ref mut statements,
                ref mut statement,
                ref mut worker,
                ..
            } = self;

            // prepare statement object (or checkout from cache)
            let stmt = prepare(worker, statements, statement, sql, persistent)?;

            // keep track of how many arguments we have bound
            let mut num_arguments = 0;

            while let Some((stmt, columns, column_names, last_row_values)) = stmt.prepare(conn)? {
                // Prepare to reset raw SQLite statement when the handle
                // is dropped. `StatementResetter` will reliably reset the
                // statement even if the stream returned from `fetch_many`
                // is dropped early.
                let resetter = StatementResetter::new(worker, stmt);

                // bind values to the statement
                num_arguments += bind(stmt, &arguments, num_arguments)?;

                loop {
                    // save the rows from the _current_ position on the statement
                    // and send them to the still-live row object
                    SqliteRow::inflate_if_needed(stmt, &*columns, last_row_values.take());

                    // invoke [sqlite3_step] on the dedicated worker thread
                    // this will move us forward one row or finish the statement
                    let s = resetter.worker.step(stmt)?;

                    match s {
                        Either::Left(changes) => {
                            let last_insert_rowid = unsafe {
                                sqlite3_last_insert_rowid(conn.as_ptr())
                            };

                            let done = SqliteQueryResult {
                                changes,
                                last_insert_rowid,
                            };

                            r#yield!(Either::Left(done));

                            break;
                        }

                        Either::Right(()) => {
                            let (row, weak_values_ref) = SqliteRow::current(
                                stmt.to_ref(conn.to_ref()),
                                columns,
                                column_names
                            );

                            let v = Either::Right(row);
                            *last_row_values = Some(weak_values_ref);

                            r#yield!(v);
                        }
                    }
                }
            }

            Ok(())
        })
    }

    fn fetch_optional< 'q, E: 'q>(
        self,
        mut query: E,
    ) ->  Result<Option<SqliteRow>, Error>
    where
        E: Execute<'q, Self::Database>,
    {
        let sql = query.sql();
        let arguments = query.take_arguments();
        let persistent = query.persistent() && arguments.is_some();
            let SqliteConnection {
                handle: ref mut conn,
                ref mut statements,
                ref mut statement,
                ref mut worker,
                ..
            } = self;

            // prepare statement object (or checkout from cache)
            let virtual_stmt = prepare(worker, statements, statement, sql, persistent)?;

            // keep track of how many arguments we have bound
            let mut num_arguments = 0;

            while let Some((stmt, columns, column_names, last_row_values)) =
                virtual_stmt.prepare(conn)?
            {
                // bind values to the statement
                num_arguments += bind(stmt, &arguments, num_arguments)?;

                // save the rows from the _current_ position on the statement
                // and send them to the still-live row object
                SqliteRow::inflate_if_needed(stmt, &*columns, last_row_values.take());

                // invoke [sqlite3_step] on the dedicated worker thread
                // this will move us forward one row or finish the statement
                match worker.step(stmt)? {
                    Either::Left(_) => (),

                    Either::Right(()) => {
                        let (row, weak_values_ref) = SqliteRow::current(
                            stmt.to_ref(self.handle.to_ref()),
                            columns,
                            column_names,
                        );

                        *last_row_values = Some(weak_values_ref);

                        virtual_stmt.reset(worker)?;
                        return Ok(Some(row));
                    }
                }
            }
            Ok(None)
    }

    fn prepare_with< 'q>(
        self,
        sql: &'q str,
        _parameters: &[SqliteTypeInfo],
    ) ->  Result<SqliteStatement<'q>, Error>
    where
    {
            let SqliteConnection {
                handle: ref mut conn,
                ref mut statements,
                ref mut statement,
                ref mut worker,
                ..
            } = self;

            // prepare statement object (or checkout from cache)
            let statement = prepare(worker, statements, statement, sql, true)?;

            let mut parameters = 0;
            let mut columns = None;
            let mut column_names = None;

            while let Some((statement, columns_, column_names_, _)) = statement.prepare(conn)? {
                parameters += statement.bind_parameter_count();

                // the first non-empty statement is chosen as the statement we pull columns from
                if !columns_.is_empty() && columns.is_none() {
                    columns = Some(Arc::clone(columns_));
                    column_names = Some(Arc::clone(column_names_));
                }
            }

            Ok(SqliteStatement {
                sql: Cow::Borrowed(sql),
                columns: columns.unwrap_or_default(),
                column_names: column_names.unwrap_or_default(),
                parameters,
            })
    }

    #[doc(hidden)]
    fn describe< 'q>(self, sql: &'q str) ->  Result<Describe<Sqlite>, Error>
    where
    {
        describe(self, sql)
    }
}

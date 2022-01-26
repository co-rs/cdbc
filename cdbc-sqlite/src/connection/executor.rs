use cdbc::describe::Describe;
use cdbc::error::Error;
use cdbc::executor::{Execute, Executor};
use crate::connection::describe::describe;
use crate::statement::{StatementHandle, VirtualStatement};
use crate::{
    Sqlite, SqliteArguments, SqliteConnection, SqliteQueryResult, SqliteRow, SqliteStatement,
    SqliteTypeInfo,
};
use either::Either;
use libsqlite3_sys::sqlite3_last_insert_rowid;
use std::borrow::Cow;
use std::sync::Arc;
use std::sync::mpsc::RecvError;
use cogo::std::io::TryStream;
use cogo::std::sync::channel;
use cogo::std::sync::channel::Receiver;
use cdbc::database::{Database, HasStatement};
use cdbc::io::chan_stream::ChanStream;
use cdbc::utils::statement_cache::StatementCache;
use crate::connection::executor_mut;


impl Executor for &mut SqliteConnection {
    type Database = Sqlite;

    fn fetch_many<'q, E: 'q>(&mut self,
                             mut query: E,
    ) -> ChanStream<Either<SqliteQueryResult, SqliteRow>>
        where
            E: Execute<'q, Self::Database>,
    {
        let sql = query.sql();
        let arguments = query.take_arguments();
        let persistent = query.persistent() && arguments.is_some();
        let s = self.worker
            .execute(sql, arguments, self.row_channel_size, persistent);
        if s.is_err() {
            let c = ChanStream::new(|sender|
                Err(s.err().unwrap())
            );
            return c;
        }
        let s = s.unwrap();
        executor_mut::sender_to_stream(s)
    }

    fn fetch_optional<'q, E: 'q>(
        &mut self,
        mut query: E,
    ) -> Result<Option<SqliteRow>, Error>
        where
            E: Execute<'q, Self::Database>,
    {
        let sql = query.sql();
        let arguments = query.take_arguments();
        let persistent = query.persistent() && arguments.is_some();
        let mut stream = self
            .worker
            .execute(sql, arguments, self.row_channel_size, persistent)?;
        let mut stream = executor_mut::sender_to_stream(stream);
        use crate::cdbc::io::chan_stream::TryStream;
        while let Some(res) = stream.try_next()? {
            if let Either::Right(row) = res {
                return Ok(Some(row));
            }
        }
        Ok(None)
    }

    fn prepare_with<'q>(
        &mut self,
        sql: &'q str,
        _parameters: &[SqliteTypeInfo],
    ) -> Result<SqliteStatement<'q>, Error>
        where
    {
        let statement = self.worker.prepare(sql)?;
        Ok(SqliteStatement {
            sql: sql.into(),
            ..statement
        })
    }

    #[doc(hidden)]
    fn describe<'q>(&mut self, sql: &'q str) -> Result<Describe<Sqlite>, Error>
    {
        self.worker.describe(sql)
    }
}
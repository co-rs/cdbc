use mco::std::sync::channel::Receiver;
use either::Either;
use cdbc::database::{Database, HasStatement};
use cdbc::{Error, Execute, Executor};
use cdbc::describe::Describe;
use cdbc::io::chan_stream::ChanStream;
use crate::{Sqlite, SqliteConnection, SqliteQueryResult, SqliteRow, SqliteStatement, SqliteTypeInfo};

pub(crate) fn sender_to_stream(arg: Receiver<Result<Either<SqliteQueryResult, SqliteRow>, Error>>) -> ChanStream<Either<SqliteQueryResult, SqliteRow>> {
    ChanStream::new(|s| {
        loop {
            match arg.recv() {
                Ok(v) => {
                    if v.is_err() {
                        return Err(v.err().unwrap());
                    } else {
                        s.send(Some(v));
                    }
                }
                Err(_) => {
                    //log::error!("{}",e);
                    // s.send(None);
                    return Ok(());
                }
            }
        }
    })
}


impl Executor for SqliteConnection {
    type Database = Sqlite;

    fn fetch_many<'q, E: 'q>(&mut self,
                             mut query: E,
    ) -> ChanStream<Either<SqliteQueryResult, SqliteRow>>
        where
            E: Execute<'q, Self::Database>,
    {
        let arguments = query.take_arguments();
        let persistent = query.persistent() && arguments.is_some();
        let s = self.worker
            .execute(query.sql(), arguments, self.row_channel_size, persistent);
        if s.is_err() {
            let c = ChanStream::new(|sender|
                Err(s.err().unwrap())
            );
            return c;
        }
        let s = s.unwrap();
        sender_to_stream(s)
    }

    fn fetch_optional<'q, E: 'q>(
        &mut self,
        mut query: E,
    ) -> Result<Option<SqliteRow>, Error>
        where
            E: Execute<'q, Self::Database>,
    {
        let arguments = query.take_arguments();
        let persistent = query.persistent() && arguments.is_some();
        let mut stream = self
            .worker
            .execute(query.sql(), arguments, self.row_channel_size, persistent)?;
        let mut stream = sender_to_stream(stream);
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
    ) -> Result<SqliteStatement, Error>
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
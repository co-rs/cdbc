use either::Either;

use crate::database::{Database, HasStatement};
use crate::describe::Describe;
use crate::error::Error;
use crate::executor::{Execute, Executor};
use crate::io::chan_stream::ChanStream;
use crate::pool::{Pool, PoolConnection};
use crate::{chan_stream};
use crate::io::chan_stream::TryStream;

impl<DB: Database> Executor for &'_ Pool<DB>
where
    for<'c> &'c mut DB::Connection: Executor< Database = DB>,
{
    type Database = DB;

    fn fetch_many<'q, E: 'q>(
        &mut self,
        query: E,
    ) -> ChanStream<Either<DB::QueryResult, DB::Row>>
    where
        E: Execute<'q, Self::Database>,
    {
        let pool = self.clone();

        chan_stream! {
            let mut conn = pool.acquire()?;
            let mut s = conn.fetch_many(query);

            while let Some(v) = s.try_next()? {
                r#yield!(v);
            }

            Ok(())
        }
    }

    fn fetch_optional<'q, E: 'q>(
        &mut self,
        query: E,
    ) ->  Result<Option<DB::Row>, Error>
    where
        E: Execute<'q, Self::Database>,
    {
        let pool = self.clone();

       pool.acquire()?.fetch_optional(query)
    }

    fn prepare_with<'q>(
        &mut self,
        sql: &'q str,
        parameters: &'q [<Self::Database as Database>::TypeInfo],
    ) ->  Result<<Self::Database as HasStatement>::Statement, Error> {
        let pool = self.clone();

         pool.acquire()?.prepare_with(sql, parameters)
    }

    #[doc(hidden)]
    fn describe<'q>(
        &mut self,
        sql: &'q str,
    ) ->  Result<Describe<Self::Database>, Error> {
        let pool = self.clone();

        pool.acquire()?.describe(sql)
    }
}


//impl to Pool<DB> owner
impl<DB: Database> Executor for Pool<DB>
    where
            for<'c> &'c mut DB::Connection: Executor< Database = DB>,
{
    type Database = DB;

    fn fetch_many<'q, E: 'q>(
        &mut self,
        query: E,
    ) -> ChanStream<Either<DB::QueryResult, DB::Row>>
        where
            E: Execute<'q, Self::Database>,
    {
        let pool = self.clone();

        chan_stream! {
            let mut conn = pool.acquire()?;
            let mut s = conn.fetch_many(query);

            while let Some(v) = s.try_next()? {
                r#yield!(v);
            }

            Ok(())
        }
    }

    fn fetch_optional<'q, E: 'q>(
        &mut self,
        query: E,
    ) ->  Result<Option<DB::Row>, Error>
        where
            E: Execute<'q, Self::Database>,
    {
        let pool = self.clone();

        pool.acquire()?.fetch_optional(query)
    }

    fn prepare_with<'q>(
        &mut self,
        sql: &'q str,
        parameters: &'q [<Self::Database as Database>::TypeInfo],
    ) ->  Result<<Self::Database as HasStatement>::Statement, Error> {
        let pool = self.clone();

        pool.acquire()?.prepare_with(sql, parameters)
    }

    #[doc(hidden)]
    fn describe<'q>(
        &mut self,
        sql: &'q str,
    ) ->  Result<Describe<Self::Database>, Error> {
        let pool = self.clone();

        pool.acquire()?.describe(sql)
    }
}
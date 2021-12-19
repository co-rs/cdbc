use either::Either;

use crate::database::{Database, HasStatement};
use crate::describe::Describe;
use crate::error::Error;
use crate::executor::{Execute, Executor};
use crate::io::chan_stream::ChanStream;
use crate::pool::Pool;
use crate::{chan_stream};
use crate::io::chan_stream::TryStream;

impl<'p, DB: Database> Executor<'p> for &'_ Pool<DB>
where
    for<'c> &'c mut DB::Connection: Executor<'c, Database = DB>,
{
    type Database = DB;

    fn fetch_many<'e, 'q: 'e, E: 'q>(
        self,
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

    fn fetch_optional<'e, 'q: 'e, E: 'q>(
        self,
        query: E,
    ) ->  Result<Option<DB::Row>, Error>
    where
        E: Execute<'q, Self::Database>,
    {
        let pool = self.clone();

       pool.acquire()?.fetch_optional(query)
    }

    fn prepare_with<'e, 'q: 'e>(
        self,
        sql: &'q str,
        parameters: &'e [<Self::Database as Database>::TypeInfo],
    ) ->  Result<<Self::Database as HasStatement<'q>>::Statement, Error> {
        let pool = self.clone();

         pool.acquire()?.prepare_with(sql, parameters)
    }

    #[doc(hidden)]
    fn describe<'e, 'q: 'e>(
        self,
        sql: &'q str,
    ) ->  Result<Describe<Self::Database>, Error> {
        let pool = self.clone();

        pool.acquire()?.describe(sql)
    }
}

// NOTE: required due to lack of lazy normalization
#[macro_export]
#[allow(unused_macros)]
macro_rules! impl_executor_for_pool_connection {
    ($DB:ident, $C:ident, $R:ident) => {
        impl<'c> cdbc::executor::Executor<'c> for &'c mut cdbc::pool::PoolConnection<$DB> {
            type Database = $DB;

            #[inline]
            fn fetch_many<'e, 'q: 'e, E: 'q>(
                self,
                query: E,
            ) -> cdbc::io::chan_stream::ChanStream<either::Either<<$DB as cdbc::database::Database>::QueryResult, $R>>
            where
                'c: 'e,
                E: cdbc::executor::Execute<'q, $DB>,
            {
                (**self).fetch_many(query)
            }

            #[inline]
            fn fetch_optional<'e, 'q: 'e, E: 'q>(
                self,
                query: E,
            ) ->  Result<Option<$R>, cdbc::error::Error>
            where
                'c: 'e,
                E: cdbc::executor::Execute<'q, $DB>,
            {
                (**self).fetch_optional(query)
            }

            #[inline]
            fn prepare_with<'e, 'q: 'e>(
                self,
                sql: &'q str,
                parameters: &'e [<$DB as cdbc::database::Database>::TypeInfo],
            ) ->
                Result<<$DB as cdbc::database::HasStatement<'q>>::Statement, cdbc::error::Error>
            where
                'c: 'e,
            {
                (**self).prepare_with(sql, parameters)
            }

            #[doc(hidden)]
            #[inline]
            fn describe<'e, 'q: 'e>(
                self,
                sql: &'q str,
            ) ->Result<cdbc::describe::Describe<$DB>, cdbc::error::Error>
            where
                'c: 'e,
            {
                (**self).describe(sql)
            }
        }
    };
}

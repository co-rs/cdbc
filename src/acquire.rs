use crate::database::Database;
use crate::error::Error;
use crate::pool::{MaybePoolConnection, Pool, PoolConnection};
use crate::transaction::Transaction;
use std::ops::{Deref, DerefMut};

pub trait Acquire<'c> {
    type Database: Database;

    type Connection: Deref<Target = <Self::Database as Database>::Connection> + DerefMut + Send;

    fn acquire(self) ->  Result<Self::Connection, Error>;

    fn begin(self) ->  Result<Transaction<'c, Self::Database>, Error>;
}

impl<'a, DB: Database> Acquire<'a> for &'_ Pool<DB> {
    type Database = DB;

    type Connection = PoolConnection<DB>;

    fn acquire(self) ->  Result<Self::Connection, Error> {
        self.acquire()
    }

    fn begin(self) ->  Result<Transaction<'a, DB>, Error> {
        let conn = self.acquire();
        Transaction::begin(MaybePoolConnection::PoolConnection(conn?))
    }
}

#[allow(unused_macros)]
macro_rules! impl_acquire {
    ($DB:ident, $C:ident) => {
        impl<'c> crate::acquire::Acquire<'c> for &'c mut $C {
            type Database = $DB;

            type Connection = &'c mut <$DB as crate::database::Database>::Connection;

            #[inline]
            fn acquire(
                self,
            ) -> Result<Self::Connection, crate::error::Error>
            {
                Ok(self)
            }

            #[inline]
            fn begin(
                self,
            ) -> Result<crate::transaction::Transaction<'c, $DB>, crate::error::Error>{
                crate::transaction::Transaction::begin(self)
            }
        }

        impl<'c> crate::acquire::Acquire<'c> for &'c mut crate::pool::PoolConnection<$DB> {
            type Database = $DB;

            type Connection = &'c mut <$DB as crate::database::Database>::Connection;

            #[inline]
            fn acquire(
                self,
            ) ->  Result<Self::Connection, crate::error::Error>
            {
                Ok(&mut **self)
            }

            #[inline]
            fn begin(
                self,
            ) -> Result<crate::transaction::Transaction<'c, $DB>, crate::error::Error>{
                crate::transaction::Transaction::begin(&mut **self)
            }
        }

        impl<'c, 't> crate::acquire::Acquire<'t>
            for &'t mut crate::transaction::Transaction<'c, $DB>
        {
            type Database = $DB;

            type Connection = &'t mut <$DB as crate::database::Database>::Connection;

            #[inline]
            fn acquire(
                self,
            ) -> Result<Self::Connection, crate::error::Error>
            {
                Ok(&mut **self)
            }

            #[inline]
            fn begin(
                self,
            ) ->Result<crate::transaction::Transaction<'t, $DB>, crate::error::Error>{
                crate::transaction::Transaction::begin(&mut **self)
            }
        }
    };
}

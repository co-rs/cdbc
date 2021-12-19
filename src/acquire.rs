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

#[macro_export]
#[allow(unused_macros)]
macro_rules! impl_acquire {
    ($DB:ident, $C:ident) => {
        impl<'c> cdbc::acquire::Acquire<'c> for &'c mut $C {
            type Database = $DB;

            type Connection = &'c mut <$DB as cdbc::database::Database>::Connection;

            #[inline]
            fn acquire(
                self,
            ) -> Result<Self::Connection, cdbc::error::Error>
            {
                Ok(self)
            }

            #[inline]
            fn begin(
                self,
            ) -> Result<cdbc::transaction::Transaction<'c, $DB>, cdbc::error::Error>{
                cdbc::transaction::Transaction::begin(self)
            }
        }

        impl<'c> cdbc::acquire::Acquire<'c> for &'c mut cdbc::pool::PoolConnection<$DB> {
            type Database = $DB;

            type Connection = &'c mut <$DB as cdbc::database::Database>::Connection;

            #[inline]
            fn acquire(
                self,
            ) ->  Result<Self::Connection, cdbc::error::Error>
            {
                Ok(&mut **self)
            }

            #[inline]
            fn begin(
                self,
            ) -> Result<cdbc::transaction::Transaction<'c, $DB>, cdbc::error::Error>{
                cdbc::transaction::Transaction::begin(&mut **self)
            }
        }

        impl<'c, 't> cdbc::acquire::Acquire<'t>
            for &'t mut cdbc::transaction::Transaction<'c, $DB>
        {
            type Database = $DB;

            type Connection = &'t mut <$DB as cdbc::database::Database>::Connection;

            #[inline]
            fn acquire(
                self,
            ) -> Result<Self::Connection, cdbc::error::Error>
            {
                Ok(&mut **self)
            }

            #[inline]
            fn begin(
                self,
            ) ->Result<cdbc::transaction::Transaction<'t, $DB>, cdbc::error::Error>{
                cdbc::transaction::Transaction::begin(&mut **self)
            }
        }
    };
}

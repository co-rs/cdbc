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

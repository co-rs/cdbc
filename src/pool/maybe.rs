use crate::database::Database;
use crate::pool::PoolConnection;
use std::ops::{Deref, DerefMut};
pub enum MaybePoolConnection<'c, DB: Database> {
    #[allow(dead_code)]
    Connection(&'c mut DB::Connection),
    PoolConnection(PoolConnection<DB>),
}

impl<'c, DB: Database> Deref for MaybePoolConnection<'c, DB> {
    type Target = DB::Connection;

    #[inline]
    fn deref(&self) -> &Self::Target {
        match self {
            MaybePoolConnection::Connection(v) => v,
            MaybePoolConnection::PoolConnection(v) => v,
        }
    }
}

impl<'c, DB: Database> DerefMut for MaybePoolConnection<'c, DB> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            MaybePoolConnection::Connection(v) => v,
            MaybePoolConnection::PoolConnection(v) => v,
        }
    }
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! impl_into_maybe_pool {
    ($DB:ident, $C:ident) => {
        // impl<'c> From<cdbc::pool::PoolConnection<$DB>>
        //     for Box<cdbc::pool::MaybePoolConnection<'c, $DB>>
        // {
        //     fn from(v: cdbc::pool::PoolConnection<$DB>) -> Self {
        //         cdbc::pool::MaybePoolConnection::PoolConnection(v)
        //     }
        // }

        impl<'c> From<&'c mut $C> for cdbc::pool::MaybePoolConnection<'c, $DB> {
            fn from(v: &'c mut $C) -> Self {
                cdbc::pool::MaybePoolConnection::Connection(v)
            }
        }
    };
}

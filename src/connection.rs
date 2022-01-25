use crate::database::{Database, HasStatementCache};
use crate::error::Error;
use crate::transaction::Transaction;
use std::fmt::Debug;
use std::str::FromStr;
use std::time::Duration;
use crate::executor::Executor;

/// Represents a single database connection.
pub trait Connection: Send+Executor {

    type Options: ConnectOptions<Connection = Self>;

    /// Explicitly close this database connection.
    ///
    /// This method is **not required** for safe and consistent operation. However, it is
    /// recommended to call it instead of letting a connection `drop` as the database backend
    /// will be faster at cleaning up resources.
    fn close(self) -> Result<(), Error>;

    /// Checks if a connection to the database is still valid.
    fn ping(&mut self) -> Result<(), Error>;

    /// Begin a new transaction or establish a savepoint within the active transaction.
    ///
    /// Returns a [`Transaction`] for controlling and tracking the new transaction.
    fn begin(&mut self) ->  Result<Transaction<'_, Self::Database>, Error>
        where
            Self: Sized;

    /// Execute the function inside a transaction.
    ///
    /// If the function returns an error, the transaction will be rolled back. If it does not
    /// return an error, the transaction will be committed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use sqlx_core::connection::Connection;
    /// use sqlx_core::error::Error;
    /// use sqlx_core::executor::Executor;
    /// use sqlx_core::postgres::{PgConnection, PgRow};
    /// use sqlx_core::query::query;
    ///
    /// # pub fn _f(conn: &mut PgConnection) -> Result<Vec<PgRow>, Error> {
    /// conn.transaction(|conn|Box::pin( || {
    ///     query("select * from ..").fetch_all(conn)
    /// }))
    /// # }
    /// ```
    fn transaction<'a, F, R, E>(&'a mut self, callback: F) ->  Result<R, E>
        where
                for<'c> F: FnOnce(&'c mut Transaction<'_, Self::Database>) ->  Result<R, E>
        + 'a
        + Send
        + Sync,
                Self: Sized,
                R: Send,
                E: From<Error> + Send,
    {
        let mut transaction = self.begin()?;
        let ret = callback(&mut transaction);
        match ret {
            Ok(ret) => {
                transaction.commit()?;
                Ok(ret)
            }
            Err(err) => {
                transaction.rollback()?;
                Err(err)
            }
        }
    }

    /// The number of statements currently cached in the connection.
    fn cached_statements_size(&self) -> usize
        where
            Self::Database: HasStatementCache,
    {
        0
    }

    /// Removes all statements from the cache, closing them on the server if
    /// needed.
    fn clear_cached_statements(&mut self) ->  Result<(), Error>
        where
            Self::Database: HasStatementCache,
    {
        Ok(())
    }

    #[doc(hidden)]
    fn flush(&mut self) -> Result<(), Error>;

    #[doc(hidden)]
    fn should_flush(&self) -> bool;

    /// Establish a new database connection.
    ///
    /// A value of [`Options`][Self::Options] is parsed from the provided connection string. This parsing
    /// is database-specific.
    #[inline]
    fn connect(url: &str) ->  Result<Self, Error>
        where
            Self: Sized,
    {
        let options = url.parse();

        Ok(Self::connect_with(&options?)?)
    }

    /// Establish a new database connection with the provided options.
    fn connect_with(options: &Self::Options) ->  Result<Self, Error>
        where
            Self: Sized,
    {
        options.connect(Duration::from_secs(10 * 60))
    }
}

pub trait ConnectOptions: 'static + Send + Sync + FromStr<Err = Error> + Debug {
    type Connection: Connection + ?Sized;

    /// Establish a new database connection with the options specified by `self`.
    fn connect(&self,d:Duration) -> Result<Self::Connection, Error>
        where
            Self::Connection: Sized;
}

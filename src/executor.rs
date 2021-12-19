use crate::database::{Database, HasArguments, HasStatement};
use crate::describe::Describe;
use crate::error::Error;
use either::Either;
use std::fmt::Debug;
use crate::{chan_stream};
use crate::io::chan_stream::{ChanStream, Stream, TryStream};

/// A type that contains or can provide a database
/// connection to use for executing queries against the database.
///
/// No guarantees are provided that successive queries run on the same
/// physical database connection.
///
/// A [`Connection`](crate::connection::Connection) is an `Executor` that guarantees that
/// successive queries are ran on the same physical database connection.
///
/// Implemented for the following:
///
///  * [`&Pool`](super::pool::Pool)
///  * [`&mut PoolConnection`](super::pool::PoolConnection)
///  * [`&mut Connection`](super::connection::Connection)
///
pub trait Executor: Send + Debug + Sized {
    type Database: Database;

    /// Execute the query and return the total number of rows affected.
    fn execute<'q,E:'q>(
        self,
        query: E,
    ) -> Result<<Self::Database as Database>::QueryResult, Error> where E: Execute<'q, Self::Database>
    {
        let mut s = self.execute_many(query);
        s.collect(|it|{
            Some(Ok(it))
        })
    }

    /// Execute multiple queries and return the rows affected from each query, in a stream.
    fn execute_many<'q,E: 'q>(
        self,
        query: E,
    ) -> ChanStream<<Self::Database as Database>::QueryResult>
        where E: Execute<'q, Self::Database>,
    {
        let mut s = self.fetch_many(query);
        s.map(|either| {
            match either {
                Either::Left(rows) => {
                    Some(rows)
                }
                Either::Right(_) => {
                    None
                }
            }
        })
    }

    /// Execute the query and return the generated results as a stream.
    fn fetch<'q, E: 'q>(
        self,
        query: E,
    ) -> ChanStream<<Self::Database as Database>::Row> where E: Execute<'q, Self::Database>,
    {
        let mut s = self.fetch_many(query);
        s.map(|either| {
            match either{
                Either::Left(rows) => {
                    None
                }
                Either::Right(row) => {
                    Some(row)
                }
            }
        })
    }

    /// Execute multiple queries and return the generated results as a stream
    /// from each query, in a stream.
    fn fetch_many<'q, E: 'q>(
        self,
        query: E,
    ) -> ChanStream<Either<<Self::Database as Database>::QueryResult, <Self::Database as Database>::Row>> where E: Execute<'q, Self::Database>;

    /// Execute the query and return all the generated results, collected into a [`Vec`].
    fn fetch_all< 'q, E: 'q>(
        self,
        query: E,
    ) -> Result<Vec<<Self::Database as Database>::Row>, Error> where  E: Execute<'q, Self::Database>
    {
         self.fetch(query).collect(|it|{
            Some(Ok(it))
        })
    }

    /// Execute the query and returns exactly one row.
    fn fetch_one<'q, E: 'q>(
        self,
        query: E,
    ) -> Result<<Self::Database as Database>::Row, Error>
        where E: Execute<'q, Self::Database>,
    {
        let row = self.fetch_optional(query)?;
        match row {
            Some(row) => Ok(row),
            None => Err(Error::RowNotFound),
        }
    }

    /// Execute the query and returns at most one row.
    fn fetch_optional<'q, E: 'q>(
        self,
        query: E,
    ) -> Result<Option<<Self::Database as Database>::Row>, Error>
        where E: Execute<'q, Self::Database>;

    /// Prepare the SQL query to inspect the type information of its parameters
    /// and results.
    ///
    /// Be advised that when using the `query`, `query_as`, or `query_scalar` functions, the query
    /// is transparently prepared and executed.
    ///
    /// This explicit API is provided to allow access to the statement metadata available after
    /// it prepared but before the first row is returned.
    #[inline]
    fn prepare<'q>(
        self,
        query: &'q str,
    ) -> Result<<Self::Database as HasStatement<'q>>::Statement, Error> {
        self.prepare_with(query, &[])
    }

    /// Prepare the SQL query, with parameter type information, to inspect the
    /// type information about its parameters and results.
    ///
    /// Only some database drivers (PostgreSQL, MSSQL) can take advantage of
    /// this extra information to influence parameter type inference.
    fn prepare_with<'q>(
        self,
        sql: &'q str,
        parameters: &'q [<Self::Database as Database>::TypeInfo],
    ) -> Result<<Self::Database as HasStatement<'q>>::Statement, Error>;

    /// Describe the SQL query and return type information about its parameters
    /// and results.
    ///
    /// This is used by compile-time verification in the query macros to
    /// power their type inference.
    #[doc(hidden)]
    fn describe(
        self,
        sql: &str,
    ) -> Result<Describe<Self::Database>, Error>;
}

/// A type that may be executed against a database connection.
///
/// Implemented for the following:
///
///  * [`&str`](std::str)
///  * [`Query`](super::query::Query)
///
pub trait Execute<'q, DB: Database>: Send + Sized {
    /// Gets the SQL that will be executed.
    fn sql(&self) -> &'q str;

    /// Gets the previously cached statement, if available.
    fn statement(&self) -> Option<&<DB as HasStatement<'q>>::Statement>;

    /// Returns the arguments to be bound against the query string.
    ///
    /// Returning `None` for `Arguments` indicates to use a "simple" query protocol and to not
    /// prepare the query. Returning `Some(Default::default())` is an empty arguments object that
    /// will be prepared (and cached) before execution.
    fn take_arguments(&mut self) -> Option<<DB as HasArguments<'q>>::Arguments>;

    /// Returns `true` if the statement should be cached.
    fn persistent(&self) -> bool;
}

// NOTE: `Execute` is explicitly not implemented for String and &String to make it slightly more
//       involved to write `conn.execute(format!("SELECT {}", val))`
impl<'q, DB: Database> Execute<'q, DB> for &'q str {
    #[inline]
    fn sql(&self) -> &'q str {
        self
    }

    #[inline]
    fn statement(&self) -> Option<&<DB as HasStatement<'q>>::Statement> {
        None
    }

    #[inline]
    fn take_arguments(&mut self) -> Option<<DB as HasArguments<'q>>::Arguments> {
        None
    }

    #[inline]
    fn persistent(&self) -> bool {
        true
    }
}

impl<'q, DB: Database> Execute<'q, DB> for (&'q str, Option<<DB as HasArguments<'q>>::Arguments>) {
    #[inline]
    fn sql(&self) -> &'q str {
        self.0
    }

    #[inline]
    fn statement(&self) -> Option<&<DB as HasStatement<'q>>::Statement> {
        None
    }

    #[inline]
    fn take_arguments(&mut self) -> Option<<DB as HasArguments<'q>>::Arguments> {
        self.1.take()
    }

    #[inline]
    fn persistent(&self) -> bool {
        true
    }
}

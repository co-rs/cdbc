use std::marker::PhantomData;

use either::Either;

use crate::arguments::{Arguments, IntoArguments};
use crate::database::{Database, HasArguments, HasStatement, HasStatementCache};
use crate::encode::Encode;
use crate::error::Error;
use crate::executor::{Execute, Executor};
use crate::io::chan_stream::ChanStream;
use crate::statement::Statement;
use crate::types::Type;
use crate::chan_stream;
use crate::io::chan_stream::TryStream;

/// Raw SQL query with bind parameters. Returned by [`query`][crate::query::query].
#[must_use = "query must be executed to affect database"]
pub struct Query<'q, DB: Database, A> {
    pub statement: Either<&'q str, &'q <DB as HasStatement<'q>>::Statement>,
    pub arguments: Option<A>,
    pub database: PhantomData<DB>,
    pub persistent: bool,
}

/// SQL query that will map its results to owned Rust types.
///
/// Returned by [`Query::try_map`], `query!()`, etc. Has most of the same methods as [`Query`] but
/// the return types are changed to reflect the mapping. However, there is no equivalent of
/// [`Query::execute`] as it doesn't make sense to map the result type and then ignore it.
///
/// [`Query::bind`] is also omitted; stylistically we recommend placing your `.bind()` calls
/// before `.try_map()`. This is also to prevent adding superfluous binds to the result of
/// `query!()` et al.
#[must_use = "query must be executed to affect database"]
pub struct Map<'q, DB: Database, F, A> {
    inner: Query<'q, DB, A>,
    mapper: F,
}

impl<'q, DB, A> Execute<'q, DB> for Query<'q, DB, A>
    where
        DB: Database,
        A: Send + IntoArguments<'q, DB>,
{
    #[inline]
    fn sql(&self) -> &'q str {
        match self.statement {
            Either::Right(ref statement) => statement.sql(),
            Either::Left(sql) => sql,
        }
    }

    fn statement(&self) -> Option<&<DB as HasStatement<'q>>::Statement> {
        match self.statement {
            Either::Right(ref statement) => Some(&statement),
            Either::Left(_) => None,
        }
    }

    #[inline]
    fn take_arguments(&mut self) -> Option<<DB as HasArguments<'q>>::Arguments> {
        self.arguments.take().map(IntoArguments::into_arguments)
    }

    #[inline]
    fn persistent(&self) -> bool {
        self.persistent
    }
}

impl<'q, DB: Database> Query<'q, DB, <DB as HasArguments<'q>>::Arguments> {
    /// Bind a value for use with this SQL query.
    ///
    /// If the number of times this is called does not match the number of bind parameters that
    /// appear in the query (`?` for most SQL flavors, `$1 .. $N` for Postgres) then an error
    /// will be returned when this query is executed.
    ///
    /// There is no validation that the value is of the type expected by the query. Most SQL
    /// flavors will perform type coercion (Postgres will return a database error).
    pub fn bind<T: 'q + Send + Encode<'q, DB> + Type<DB>>(mut self, value: T) -> Self {
        if let Some(arguments) = &mut self.arguments {
            arguments.add(value);
        }

        self
    }
}

impl<'q, DB, A> Query<'q, DB, A>
    where
        DB: Database + HasStatementCache,
{
    /// If `true`, the statement will get prepared once and cached to the
    /// connection's statement cache.
    ///
    /// If queried once with the flag set to `true`, all subsequent queries
    /// matching the one with the flag will use the cached statement until the
    /// cache is cleared.
    ///
    /// Default: `true`.
    pub fn persistent(mut self, value: bool) -> Self {
        self.persistent = value;
        self
    }
}

impl<'q, DB, A: Send> Query<'q, DB, A>
    where
        DB: Database,
        A: 'q + IntoArguments<'q, DB>,
{
    /// Map each row in the result to another type.
    ///
    /// See [`try_map`](Query::try_map) for a fallible version of this method.
    ///
    /// The [`query_as`](super::query_as::query_as) method will construct a mapped query using
    /// a [`FromRow`](super::from_row::FromRow) implementation.
    #[inline]
    pub fn map<F, O>(
        self,
        mut f: F,
    ) -> Map<'q, DB, impl FnMut(DB::Row) -> Result<O, Error> + Send, A>
        where
            F: FnMut(DB::Row) -> O + Send,

    {
        self.try_map(move |row| Ok(f(row)))
    }

    /// Map each row in the result to another type.
    ///
    /// The [`query_as`](super::query_as::query_as) method will construct a mapped query using
    /// a [`FromRow`](super::from_row::FromRow) implementation.
    #[inline]
    pub fn try_map<F, O>(self, f: F) -> Map<'q, DB, F, A>
        where
            F: FnMut(DB::Row) -> Result<O, Error> + Send,

    {
        Map {
            inner: self,
            mapper: f,
        }
    }

    /// Execute the query and return the total number of rows affected.
    #[inline]
    pub fn execute<'c, E>(self, mut executor: E) -> Result<DB::QueryResult, Error>
        where E: Executor<Database=DB>,
    {
        executor.execute(self)
    }

    /// Execute multiple queries and return the rows affected from each query, in a stream.
    #[inline]
    pub fn execute_many<'c, E>(
        self,
        mut executor: E,
    ) -> ChanStream<DB::QueryResult>
        where E: Executor<Database=DB>,
    {
        executor.execute_many(self)
    }

    /// Execute the query and return the generated results as a stream.
    #[inline]
    pub fn fetch<'c, E>(self, mut executor: E) -> ChanStream<DB::Row>
        where E: Executor<Database=DB>,
    {
        executor.fetch(self)
    }

    /// Execute multiple queries and return the generated results as a stream
    /// from each query, in a stream.
    #[inline]
    pub fn fetch_many<'c, E>(
        self,
        mut executor: E,
    ) -> ChanStream<Either<DB::QueryResult, DB::Row>>
        where E: Executor<Database=DB>,
    {
        executor.fetch_many(self)
    }

    /// Execute the query and return all the generated results, collected into a [`Vec`].
    #[inline]
    pub fn fetch_all<'c, E>(self, mut executor: E) -> Result<Vec<DB::Row>, Error>
        where E: Executor<Database=DB>,
    {
        executor.fetch_all(self)
    }

    /// Execute the query and returns exactly one row.
    #[inline]
    pub fn fetch_one<'c, E>(self, mut executor: E) -> Result<DB::Row, Error>
        where E: Executor<Database=DB>,
    {
        executor.fetch_one(self)
    }

    /// Execute the query and returns at most one row.
    #[inline]
    pub fn fetch_optional<'c, E>(self, mut executor: E) -> Result<Option<DB::Row>, Error>
        where E: Executor<Database=DB>,
    {
        executor.fetch_optional(self)
    }
}

impl<'q, DB, F: Send, A: Send> Execute<'q, DB> for Map<'q, DB, F, A>
    where
        DB: Database,
        A: IntoArguments<'q, DB>,
{
    #[inline]
    fn sql(&self) -> &'q str {
        self.inner.sql()
    }

    #[inline]
    fn statement(&self) -> Option<&<DB as HasStatement<'q>>::Statement> {
        self.inner.statement()
    }

    #[inline]
    fn take_arguments(&mut self) -> Option<<DB as HasArguments<'q>>::Arguments> {
        self.inner.take_arguments()
    }

    #[inline]
    fn persistent(&self) -> bool {
        self.inner.arguments.is_some()
    }
}

impl<'q, DB, F, O, A> Map<'q, DB, F, A>
    where
        DB: Database,
        F: FnMut(DB::Row) -> Result<O, Error> + Send,
        O: Send,
        A: 'q + Send + IntoArguments<'q, DB>,
{
    /// Map each row in the result to another type.
    ///
    /// See [`try_map`](Map::try_map) for a fallible version of this method.
    ///
    /// The [`query_as`](super::query_as::query_as) method will construct a mapped query using
    /// a [`FromRow`](super::from_row::FromRow) implementation.
    #[inline]
    pub fn map<G, P>(
        self,
        mut g: G,
    ) -> Map<'q, DB, impl FnMut(DB::Row) -> Result<P, Error> + Send, A>
        where
            G: FnMut(O) -> P + Send,
    {
        self.try_map(move |data| Ok(g(data)))
    }

    /// Map each row in the result to another type.
    ///
    /// The [`query_as`](super::query_as::query_as) method will construct a mapped query using
    /// a [`FromRow`](super::from_row::FromRow) implementation.
    #[inline]
    pub fn try_map<G, P>(
        self,
        mut g: G,
    ) -> Map<'q, DB, impl FnMut(DB::Row) -> Result<P, Error> + Send, A>
        where
            G: FnMut(O) -> Result<P, Error> + Send
    {
        let mut f = self.mapper;
        Map {
            inner: self.inner,
            mapper: move |row| f(row).and_then(|o| g(o)),
        }
    }

    /// Execute the query and return the generated results as a stream.
    pub fn fetch<'c, E>(self, mut executor: E) -> ChanStream<O>
        where E: 'c + Executor<Database=DB>,
              DB: 'c,
              F: 'c,
              O: 'c,
    {
        self.fetch_many(executor)
            .map(|step| {
                match step {
                    Either::Left(_) => None,
                    Either::Right(o) => Some(o),
                }
            })
    }

    /// Execute multiple queries and return the generated results as a stream
    /// from each query, in a stream.
    pub fn fetch_many<'c, E>(
        mut self,
        mut executor: E,
    ) -> ChanStream<Either<DB::QueryResult, O>>
        where E: 'c + Executor<Database=DB>,
              DB: 'c,
              F: 'c,
              O: 'c,
    {
        chan_stream! {
            let mut s = executor.fetch_many(self.inner);

            while let Some(v) = s.try_next()? {
                r#yield!(match v {
                    Either::Left(v) => Either::Left(v),
                    Either::Right(row) => {
                        Either::Right((self.mapper)(row)?)
                    }
                });
            }

            Ok(())
        }
    }

    /// Execute the query and return all the generated results, collected into a [`Vec`].
    pub fn fetch_all<'c, E>(self, mut executor: E) -> Result<Vec<O>, Error>
        where E: 'c + Executor<Database=DB>,
              DB: 'c,
              F: 'c,
              O: 'c,
    {
        self.fetch(executor).collect(|it| {
            Some(Ok(it))
        })
    }

    /// Execute the query and returns exactly one row.
    pub fn fetch_one<'c, E>(self, mut executor: E) -> Result<O, Error>
        where E: 'c + Executor<Database=DB>,
              DB: 'c,
              F: 'c,
              O: 'c,
    {
        self.fetch_optional(executor)
            .and_then(|row| match row {
                Some(row) => Ok(row),
                None => Err(Error::RowNotFound),
            })
    }

    /// Execute the query and returns at most one row.
    pub fn fetch_optional<'c, E>(mut self, mut executor: E) -> Result<Option<O>, Error>
        where E: 'c + Executor<Database=DB>,
              DB: 'c,
              F: 'c,
              O: 'c,
    {
        let row = executor.fetch_optional(self.inner)?;

        if let Some(row) = row {
            (self.mapper)(row).map(Some)
        } else {
            Ok(None)
        }
    }
}

// Make a SQL query from a statement.
pub fn query_statement<'q, DB>(
    statement: &'q <DB as HasStatement<'q>>::Statement,
) -> Query<'q, DB, <DB as HasArguments<'_>>::Arguments>
    where
        DB: Database,
{
    Query {
        database: PhantomData,
        arguments: Some(Default::default()),
        statement: Either::Right(statement),
        persistent: true,
    }
}

// Make a SQL query from a statement, with the given arguments.
pub fn query_statement_with<'q, DB, A>(
    statement: &'q <DB as HasStatement<'q>>::Statement,
    arguments: A,
) -> Query<'q, DB, A>
    where
        DB: Database,
        A: IntoArguments<'q, DB>,
{
    Query {
        database: PhantomData,
        arguments: Some(arguments),
        statement: Either::Right(statement),
        persistent: true,
    }
}

/// Make a SQL query.
pub fn query<DB>(sql: &str) -> Query<'_, DB, <DB as HasArguments<'_>>::Arguments>
    where
        DB: Database,
{
    Query {
        database: PhantomData,
        arguments: Some(Default::default()),
        statement: Either::Left(sql),
        persistent: true,
    }
}

/// Make a SQL query, with the given arguments.
pub fn query_with<'q, DB, A>(sql: &'q str, arguments: A) -> Query<'q, DB, A>
    where
        DB: Database,
        A: IntoArguments<'q, DB>,
{
    Query {
        database: PhantomData,
        arguments: Some(arguments),
        statement: Either::Left(sql),
        persistent: true,
    }
}

#[macro_export]
macro_rules! query {
    ($sql:expr) => {
        $crate::query($sql)
    };
    ($sql:expr,$($arg:tt$(,)?)*) => {
        {
            let mut q = $crate::query($sql);
            $(q = q.bind($arg);)*
            q
        }
    };
}
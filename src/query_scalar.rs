use either::Either;
use crate::arguments::IntoArguments;
use crate::database::{Database, HasArguments, HasStatement, HasStatementCache};
use crate::encode::Encode;
use crate::error::Error;
use crate::executor::{Execute, Executor};
use crate::from_row::FromRow;
use crate::io::chan_stream::ChanStream;
use crate::query_as::{
    query_as, query_as_with, query_statement_as, query_statement_as_with, QueryAs,
};
use crate::types::Type;

/// Raw SQL query with bind parameters, mapped to a concrete type using [`FromRow`] on `(O,)`.
/// Returned from [`query_scalar`].
#[must_use = "query must be executed to affect database"]
pub struct QueryScalar<DB: Database, O, A> {
    inner: QueryAs< DB, (O,), A>,
}

impl<'q, DB: Database, O: Send, A: Send> Execute<'q, DB> for QueryScalar< DB, O, A>
where
    A: 'q + IntoArguments<'q, DB>,
{
    #[inline]
    fn sql(&self) -> &str {
        self.inner.sql()
    }

    fn statement(&self) -> Option<&<DB as HasStatement>::Statement> {
        self.inner.statement()
    }

    #[inline]
    fn take_arguments(&mut self) -> Option<<DB as HasArguments<'q>>::Arguments> {
        self.inner.take_arguments()
    }

    #[inline]
    fn persistent(&self) -> bool {
        self.inner.persistent()
    }
}

impl<'q, DB: Database, O> QueryScalar< DB, O, <DB as HasArguments<'q>>::Arguments> {
    /// Bind a value for use with this SQL query.
    ///
    /// See [`Query::bind`](crate::query::Query::bind).
    pub fn bind<T: 'q + Send + Encode<'q, DB> + Type<DB>>(mut self, value: T) -> Self {
        self.inner = self.inner.bind(value);
        self
    }
}

impl<'q, DB, O, A> QueryScalar< DB, O, A>
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
        self.inner = self.inner.persistent(value);
        self
    }
}

// FIXME: This is very close, nearly 1:1 with `Map`
// noinspection DuplicatedCode
impl<'q, DB, O, A> QueryScalar< DB, O, A>
where
    DB: Database,
    O: Send,
    A: 'q + IntoArguments<'q, DB>,
    (O,): Send +  for<'r> FromRow<'r, DB::Row>,
{
    /// Execute the query and return the generated results as a stream.
    #[inline]
    pub fn fetch<'e, 'c: 'e, E>(self, executor: E) -> ChanStream<O>
    where
        'q: 'e,
        E: 'e + Executor< Database = DB>,
        DB: 'e,
        A: 'e,
        O: 'e,
    {
        //self.inner.fetch(executor).map_ok(|it| it.0).boxed()
        self.inner.fetch(executor).map(|v|{
            Some(v.0)
        })
    }

    /// Execute multiple queries and return the generated results as a stream
    /// from each query, in a stream.
    #[inline]
    pub fn fetch_many<'e, 'c: 'e, E>(
        self,
        executor: E,
    ) -> ChanStream<Either<DB::QueryResult, O>>
    where
        'q: 'e,
        E: 'e + Executor< Database = DB>,
        DB: 'e,
        A: 'e,
        O: 'e,
    {
        self.inner
            .fetch_many(executor)
            .map(|v| {
                Some(v.map_right(|it| it.0))
            })
    }

    /// Execute the query and return all the generated results, collected into a [`Vec`].
    #[inline]
    pub fn fetch_all<'e, 'c: 'e, E>(self, executor: E) -> Result<Vec<O>, Error>
    where
        'q: 'e,
        E: 'e + Executor< Database = DB>,
        DB: 'e,
        (O,): 'e,
        A: 'e,
    {
        self.inner
            .fetch(executor)
            .map(|it| Some(it.0))
            .collect(|v|{
                Some(Ok(v))
            })
            
    }

    /// Execute the query and returns exactly one row.
    #[inline]
    pub fn fetch_one<'e, 'c: 'e, E>(self, executor: E) -> Result<O, Error>
    where
        'q: 'e,
        E: 'e + Executor< Database = DB>,
        DB: 'e,
        O: 'e,
        A: 'e,
    {
        self.inner.fetch_one(executor).map(|it| it.0)
    }

    /// Execute the query and returns at most one row.
    #[inline]
    pub fn fetch_optional<'e, 'c: 'e, E>(self, executor: E) -> Result<Option<O>, Error>
    where
        'q: 'e,
        E: 'e + Executor< Database = DB>,
        DB: 'e,
        O: 'e,
        A: 'e,
    {
        Ok(self.inner.fetch_optional(executor)?.map(|it| it.0))
    }
}

/// Make a SQL query that is mapped to a single concrete type
/// using [`FromRow`].
#[inline]
pub fn query_scalar<'q, DB, O>(
    sql: &'q str,
) -> QueryScalar< DB, O, <DB as HasArguments<'q>>::Arguments>
where
    DB: Database,
    (O,): for<'r> FromRow<'r, DB::Row>,
{
    QueryScalar {
        inner: query_as(sql),
    }
}

/// Make a SQL query, with the given arguments, that is mapped to a single concrete type
/// using [`FromRow`].
#[inline]
pub fn query_scalar_with<'q, DB, O, A>(sql: &'q str, arguments: A) -> QueryScalar< DB, O, A>
where
    DB: Database,
    A: IntoArguments<'q, DB>,
    (O,): for<'r> FromRow<'r, DB::Row>,
{
    QueryScalar {
        inner: query_as_with(sql, arguments),
    }
}

// Make a SQL query from a statement, that is mapped to a concrete value.
pub fn query_statement_scalar<'q, DB, O>(
    statement: <DB as HasStatement>::Statement,
) -> QueryScalar< DB, O, <DB as HasArguments<'q>>::Arguments>
where
    DB: Database,
    (O,): for<'r> FromRow<'r, DB::Row>,
{
    QueryScalar {
        inner: query_statement_as(statement),
    }
}

// Make a SQL query from a statement, with the given arguments, that is mapped to a concrete value.
pub fn query_statement_scalar_with<'q, DB, O, A>(
    statement:  <DB as HasStatement>::Statement,
    arguments: A,
) -> QueryScalar< DB, O, A>
where
    DB: Database,
    A: IntoArguments<'q, DB>,
    (O,): for<'r> FromRow<'r, DB::Row>,
{
    QueryScalar {
        inner: query_statement_as_with(statement, arguments),
    }
}

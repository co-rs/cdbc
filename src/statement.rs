use std::borrow::Cow;
use crate::arguments::IntoArguments;
use crate::column::ColumnIndex;
use crate::database::{Database, HasArguments, HasStatement};
use crate::error::Error;
use crate::from_row::FromRow;
use crate::query::Query;
use crate::query_as::QueryAs;
use crate::query_scalar::QueryScalar;
use either::Either;

/// An explicitly prepared statement.
///
/// Statements are prepared and cached by default, per connection. This type allows you to
/// look at that cache in-between the statement being prepared and it being executed. This contains
/// the expected columns to be returned and the expected parameter types (if available).
///
/// Statements can be re-used with any connection and on first-use it will be re-prepared and
/// cached within the connection.
pub trait Statement: Send + Sync {
    type Database: Database;

    /// Creates an owned statement from this statement reference. This copies
    /// the original SQL text.
    fn to_owned(&self) -> <Self::Database as HasStatement>::Statement;

    /// Get the original SQL text used to create this statement.
    fn sql(&self) -> &str;

    /// get mut Cow str
    fn sql_mut(&mut self) -> &mut String;
    /// Get the expected parameters for this statement.
    ///
    /// The information returned depends on what is available from the driver. SQLite can
    /// only tell us the number of parameters. PostgreSQL can give us full type information.
    fn parameters(&self) -> Option<Either<&[<Self::Database as Database>::TypeInfo], usize>>;

    /// Get the columns expected to be returned by executing this statement.
    fn columns(&self) -> &[<Self::Database as Database>::Column];

    /// Gets the column information at `index`.
    ///
    /// A string index can be used to access a column by name and a `usize` index
    /// can be used to access a column by position.
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    /// See [`try_column`](Self::try_column) for a non-panicking version.
    fn column<I>(&self, index: I) -> &<Self::Database as Database>::Column
        where
            I: ColumnIndex<Self>,
    {
        self.try_column(index).unwrap()
    }

    /// Gets the column information at `index` or `None` if out of bounds.
    fn try_column<I>(&self, index: I) -> Result<&<Self::Database as Database>::Column, Error>
        where
            I: ColumnIndex<Self>,
    {
        Ok(&self.columns()[index.index(self)?])
    }

    fn query(self) -> Query<Self::Database, <Self::Database as HasArguments<'static>>::Arguments>;

    fn query_with<'s, A>(self, arguments: A) -> Query<Self::Database, A>
        where
            A: IntoArguments<'s, Self::Database>;

    fn query_as<O>(
        self,
    ) -> QueryAs<Self::Database, O, <Self::Database as HasArguments<'static>>::Arguments>
        where
            O: for<'r> FromRow<'r, <Self::Database as Database>::Row>;

    fn query_as_with<'s, O, A>(self, arguments: A) -> QueryAs<Self::Database, O, A>
        where
            O: for<'r> FromRow<'r, <Self::Database as Database>::Row>,
            A: IntoArguments<'s, Self::Database>;

    fn query_scalar<O>(
        self,
    ) -> QueryScalar<Self::Database, O, <Self::Database as HasArguments<'static>>::Arguments>
        where
            (O, ): for<'r> FromRow<'r, <Self::Database as Database>::Row>;

    fn query_scalar_with<'s, O, A>(self, arguments: A) -> QueryScalar<Self::Database, O, A>
        where
            (O, ): for<'r> FromRow<'r, <Self::Database as Database>::Row>,
            A: IntoArguments<'s, Self::Database>;
}

#[macro_export]
macro_rules! impl_statement_query {
    ($A:ty) => {
        #[inline]
        fn query(self) -> cdbc::query::Query< Self::Database, $A> {
            cdbc::query::query_statement(self)
        }

        #[inline]
        fn query_with<'s, A>(self, arguments: A) -> cdbc::query::Query<Self::Database, A>
        where
            A: cdbc::arguments::IntoArguments<'s, Self::Database>,
        {
            cdbc::query::query_statement_with(self, arguments)
        }

        #[inline]
        fn query_as<O>(
            self,
        ) -> cdbc::query_as::QueryAs<
            Self::Database,
            O,
            <Self::Database as cdbc::database::HasArguments<'static>>::Arguments,
        >
        where
            O: for<'r> cdbc::from_row::FromRow<
                'r,
                <Self::Database as cdbc::database::Database>::Row,
            >,
        {
            cdbc::query_as::query_statement_as(self)
        }

        #[inline]
        fn query_as_with<'s, O, A>(
           self,
            arguments: A,
        ) -> cdbc::query_as::QueryAs<Self::Database, O, A>
        where
            O: for<'r> cdbc::from_row::FromRow<
                'r,
                <Self::Database as cdbc::database::Database>::Row,
            >,
            A: cdbc::arguments::IntoArguments<'s, Self::Database>,
        {
            cdbc::query_as::query_statement_as_with(self, arguments)
        }

        #[inline]
        fn query_scalar<O>(
            self,
        ) -> cdbc::query_scalar::QueryScalar<
            Self::Database,
            O,
            <Self::Database as cdbc::database::HasArguments<'static>>::Arguments,
        >
        where
            (O,): for<'r> cdbc::from_row::FromRow<
                'r,
                <Self::Database as cdbc::database::Database>::Row,
            >,
        {
            cdbc::query_scalar::query_statement_scalar(self)
        }

        #[inline]
        fn query_scalar_with<'s, O, A>(
            self,
            arguments: A,
        ) -> cdbc::query_scalar::QueryScalar<Self::Database, O, A>
        where
            (O,): for<'r> cdbc::from_row::FromRow<
                'r,
                <Self::Database as cdbc::database::Database>::Row,
            >,
            A: cdbc::arguments::IntoArguments<'s, Self::Database>,
        {
            cdbc::query_scalar::query_statement_scalar_with(self, arguments)
        }
    };
}

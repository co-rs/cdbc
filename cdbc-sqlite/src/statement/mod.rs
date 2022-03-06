use cdbc::column::ColumnIndex;
use cdbc::error::Error;
use crate::{Sqlite, SqliteArguments, SqliteColumn, SqliteTypeInfo};
use cdbc::statement::Statement;
use cdbc::HashMap;
use either::Either;
use std::sync::Arc;
use cdbc::ustr::UStr;

mod handle;
mod r#virtual;

pub use handle::StatementHandle;
pub use r#virtual::VirtualStatement;

#[derive(Debug, Clone)]
#[allow(clippy::rc_buffer)]
pub struct SqliteStatement {
    pub sql: String,
    pub parameters: usize,
    pub columns: Arc<Vec<SqliteColumn>>,
    pub column_names: Arc<HashMap<UStr, usize>>,
}

impl Statement for SqliteStatement {
    type Database = Sqlite;

    fn to_owned(&self) -> SqliteStatement {
        SqliteStatement {
            sql: self.sql.clone(),
            parameters: self.parameters,
            columns: Arc::clone(&self.columns),
            column_names: Arc::clone(&self.column_names),
        }
    }

    fn sql(&self) -> &str {
        &self.sql
    }

    fn sql_mut(&mut self) -> &mut String {
        &mut self.sql
    }

    fn parameters(&self) -> Option<Either<&[SqliteTypeInfo], usize>> {
        Some(Either::Right(self.parameters))
    }

    fn columns(&self) -> &[SqliteColumn] {
        &self.columns
    }

    #[inline]
    fn query(self) -> cdbc::query::Query< Self::Database, SqliteArguments<'static>> {
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
}

impl ColumnIndex<SqliteStatement> for &'_ str {
    fn index(&self, statement: &SqliteStatement) -> Result<usize, Error> {
        statement
            .column_names
            .get(*self)
            .ok_or_else(|| Error::ColumnNotFound((*self).into()))
            .map(|v| *v)
    }
}
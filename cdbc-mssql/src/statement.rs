use cdbc::column::ColumnIndex;
use cdbc::utils::ustr::UStr;
use crate::{Mssql, MssqlArguments, MssqlColumn, MssqlTypeInfo};
use cdbc::statement::Statement;
use cdbc::{Error, HashMap};
use either::Either;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct MssqlStatement {
    pub sql: String,
    pub metadata: Arc<MssqlStatementMetadata>,
}

#[derive(Debug, Default, Clone)]
pub struct MssqlStatementMetadata {
    pub columns: Vec<MssqlColumn>,
    pub column_names: HashMap<UStr, usize>,
}

impl Statement for MssqlStatement {
    type Database = Mssql;

    fn to_owned(&self) -> MssqlStatement {
        MssqlStatement {
            sql: self.sql.clone(),
            metadata: self.metadata.clone(),
        }
    }

    fn sql(&self) -> &str {
        &self.sql
    }

    fn sql_mut(&mut self) -> &mut String {
        &mut self.sql
    }

    fn parameters(&self) -> Option<Either<&[MssqlTypeInfo], usize>> {
        None
    }

    fn columns(&self) -> &[MssqlColumn] {
        &self.metadata.columns
    }

    #[inline]
    fn query(self) -> cdbc::query::Query< Self::Database, MssqlArguments> {
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
        <Self::Database as cdbc::database::HasArguments::<'static>>::Arguments>
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

impl ColumnIndex<MssqlStatement> for &'_ str {
    fn index(&self, statement: &MssqlStatement) -> Result<usize, Error> {
        statement
            .metadata
            .column_names
            .get(*self)
            .ok_or_else(|| Error::ColumnNotFound((*self).into()))
            .map(|v| *v)
    }
}

use cdbc::column::ColumnIndex;
use cdbc::error::Error;
use crate::{Sqlite, SqliteArguments, SqliteColumn, SqliteTypeInfo};
use cdbc::statement::Statement;
use cdbc::HashMap;
use either::Either;
use std::borrow::Cow;
use std::sync::Arc;
use cdbc::ustr::UStr;

mod handle;
mod r#virtual;

pub use handle::StatementHandle;
pub use r#virtual::VirtualStatement;

#[derive(Debug, Clone)]
#[allow(clippy::rc_buffer)]
pub struct SqliteStatement<'q> {
    pub sql: Cow<'q, str>,
    pub parameters: usize,
    pub columns: Arc<Vec<SqliteColumn>>,
    pub column_names: Arc<HashMap<UStr, usize>>,
}

impl<'q> Statement<'q> for SqliteStatement<'q> {
    type Database = Sqlite;

    fn to_owned(&self) -> SqliteStatement<'static> {
        SqliteStatement::<'static> {
            sql: Cow::Owned(self.sql.clone().into_owned()),
            parameters: self.parameters,
            columns: Arc::clone(&self.columns),
            column_names: Arc::clone(&self.column_names),
        }
    }

    fn sql(&self) -> &str {
        &self.sql
    }

    fn parameters(&self) -> Option<Either<&[SqliteTypeInfo], usize>> {
        Some(Either::Right(self.parameters))
    }

    fn columns(&self) -> &[SqliteColumn] {
        &self.columns
    }

    impl_statement_query!(SqliteArguments<'_>);
}

impl ColumnIndex<SqliteStatement<'_>> for &'_ str {
    fn index(&self, statement: &SqliteStatement<'_>) -> Result<usize, Error> {
        statement
            .column_names
            .get(*self)
            .ok_or_else(|| Error::ColumnNotFound((*self).into()))
            .map(|v| *v)
    }
}

#[cfg(feature = "any")]
impl<'q> From<SqliteStatement<'q>> for crate::any::AnyStatement<'q> {
    #[inline]
    fn from(statement: SqliteStatement<'q>) -> Self {
        crate::any::AnyStatement::<'q> {
            columns: statement
                .columns
                .iter()
                .map(|col| col.clone().into())
                .collect(),
            column_names: statement.column_names,
            parameters: Some(Either::Right(statement.parameters)),
            sql: statement.sql,
        }
    }
}

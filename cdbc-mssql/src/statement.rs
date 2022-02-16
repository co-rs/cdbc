use cdbc::column::ColumnIndex;
use cdbc::utils::ustr::UStr;
use crate::{Mssql, MssqlArguments, MssqlColumn, MssqlTypeInfo};
use cdbc::statement::Statement;
use cdbc::{Error, HashMap};
use either::Either;
use std::borrow::Cow;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct MssqlStatement<'q> {
    pub sql: Cow<'q, str>,
    pub metadata: Arc<MssqlStatementMetadata>,
}

#[derive(Debug, Default, Clone)]
pub struct MssqlStatementMetadata {
    pub columns: Vec<MssqlColumn>,
    pub column_names: HashMap<UStr, usize>,
}

impl<'q> Statement<'q> for MssqlStatement<'q> {
    type Database = Mssql;

    fn to_owned(&self) -> MssqlStatement<'static> {
        MssqlStatement::<'static> {
            sql: Cow::Owned(self.sql.clone().into_owned()),
            metadata: self.metadata.clone(),
        }
    }

    fn sql(&self) -> &str {
        &self.sql
    }

    fn parameters(&self) -> Option<Either<&[MssqlTypeInfo], usize>> {
        None
    }

    fn columns(&self) -> &[MssqlColumn] {
        &self.metadata.columns
    }

    impl_statement_query!(MssqlArguments);
}

impl ColumnIndex<MssqlStatement<'_>> for &'_ str {
    fn index(&self, statement: &MssqlStatement<'_>) -> Result<usize, Error> {
        statement
            .metadata
            .column_names
            .get(*self)
            .ok_or_else(|| Error::ColumnNotFound((*self).into()))
            .map(|v| *v)
    }
}

#[cfg(feature = "any")]
impl<'q> From<MssqlStatement<'q>> for crate::any::AnyStatement<'q> {
    #[inline]
    fn from(statement: MssqlStatement<'q>) -> Self {
        crate::any::AnyStatement::<'q> {
            columns: statement
                .metadata
                .columns
                .iter()
                .map(|col| col.clone().into())
                .collect(),
            column_names: std::sync::Arc::new(statement.metadata.column_names.clone()),
            parameters: None,
            sql: statement.sql,
        }
    }
}

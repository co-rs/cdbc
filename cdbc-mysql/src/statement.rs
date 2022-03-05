use super::MySqlColumn;
use cdbc::column::ColumnIndex;
use cdbc::error::Error;

use crate::{MySql, MySqlArguments, MySqlTypeInfo};
use cdbc::statement::Statement;
use cdbc::HashMap;
use either::Either;
use std::borrow::Cow;
use std::sync::Arc;
use cdbc::utils::ustr::UStr;

#[derive(Debug, Clone)]
pub struct MySqlStatement {
    pub(crate) sql: String,
    pub(crate) metadata: MySqlStatementMetadata,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct MySqlStatementMetadata {
    pub(crate) columns: Arc<Vec<MySqlColumn>>,
    pub(crate) column_names: Arc<HashMap<UStr, usize>>,
    pub(crate) parameters: usize,
}

impl Statement for MySqlStatement {
    type Database = MySql;

    fn to_owned(&self) -> MySqlStatement {
        MySqlStatement {
            sql: self.sql.clone(),
            metadata: self.metadata.clone(),
        }
    }

    fn sql(&self) -> &str {
        self.sql.as_str()
    }

    fn sql_mut(&mut self) -> &mut String {
        &mut self.sql
    }

    fn parameters(&self) -> Option<Either<&[MySqlTypeInfo], usize>> {
        Some(Either::Right(self.metadata.parameters))
    }

    fn columns(&self) -> &[MySqlColumn] {
        &self.metadata.columns
    }

    impl_statement_query!(MySqlArguments);
}

impl ColumnIndex<MySqlStatement> for &'_ str {
    fn index(&self, statement: &MySqlStatement) -> Result<usize, Error> {
        statement
            .metadata
            .column_names
            .get(*self)
            .ok_or_else(|| Error::ColumnNotFound((*self).into()))
            .map(|v| *v)
    }
}
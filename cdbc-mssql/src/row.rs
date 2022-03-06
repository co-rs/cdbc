use cdbc::column::ColumnIndex;
use cdbc::utils::ustr::UStr;
use crate::protocol::row::Row as ProtocolRow;
use crate::{Mssql, MssqlColumn, MssqlValueRef};
use cdbc::row::Row;
use cdbc::{Error, HashMap};
use std::sync::Arc;

pub struct MssqlRow {
    pub row: ProtocolRow,
    pub columns: Arc<Vec<MssqlColumn>>,
    pub column_names: Arc<HashMap<UStr, usize>>,
}

impl Row for MssqlRow {
    type Database = Mssql;

    fn columns(&self) -> &[MssqlColumn] {
        &*self.columns
    }

    fn try_get_raw<I>(&self, index: I) -> Result<MssqlValueRef<'_>, Error>
    where
        I: ColumnIndex<Self>,
    {
        let index = index.index(self)?;
        let value = MssqlValueRef {
            data: self.row.values[index].as_ref(),
            type_info: self.row.column_types[index].clone(),
        };

        Ok(value)
    }
}

impl ColumnIndex<MssqlRow> for &'_ str {
    fn index(&self, row: &MssqlRow) -> Result<usize, Error> {
        row.column_names
            .get(*self)
            .ok_or_else(|| Error::ColumnNotFound((*self).into()))
            .map(|v| *v)
    }
}

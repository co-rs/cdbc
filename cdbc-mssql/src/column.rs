use cdbc::column::Column;
use cdbc::utils::ustr::UStr;
use crate::protocol::col_meta_data::{ColumnData, Flags};
use crate::{Mssql, MssqlTypeInfo};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "offline", derive(serde::Serialize, serde::Deserialize))]
pub struct MssqlColumn {
    pub ordinal: usize,
    pub name: UStr,
    pub type_info: MssqlTypeInfo,
    pub flags: Flags,
}

impl MssqlColumn {
    pub fn new(meta: ColumnData, ordinal: usize) -> Self {
        Self {
            name: UStr::from(meta.col_name),
            type_info: MssqlTypeInfo(meta.type_info),
            ordinal,
            flags: meta.flags,
        }
    }
}

impl Column for MssqlColumn {
    type Database = Mssql;

    fn ordinal(&self) -> usize {
        self.ordinal
    }

    fn name(&self) -> &str {
        &*self.name
    }

    fn type_info(&self) -> &MssqlTypeInfo {
        &self.type_info
    }
}

#[cfg(feature = "any")]
impl From<MssqlColumn> for crate::any::AnyColumn {
    #[inline]
    fn from(column: MssqlColumn) -> Self {
        crate::any::AnyColumn {
            type_info: column.type_info.clone().into(),
            kind: crate::any::column::AnyColumnKind::Mssql(column),
        }
    }
}

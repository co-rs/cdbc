use cdbc::column::Column;
use crate::protocol::text::ColumnFlags;
use crate::{MySql, MySqlTypeInfo};
use cdbc::utils::ustr::UStr;

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct MySqlColumn {
    pub(crate) ordinal: usize,
    pub(crate) name: UStr,
    pub(crate) type_info: MySqlTypeInfo,

    #[serde(skip)]
    pub(crate) flags: Option<ColumnFlags>,
}


impl Column for MySqlColumn {
    type Database = MySql;

    fn ordinal(&self) -> usize {
        self.ordinal
    }

    fn name(&self) -> &str {
        &*self.name
    }

    fn type_info(&self) -> &MySqlTypeInfo {
        &self.type_info
    }
}

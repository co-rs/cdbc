use cdbc::column::Column;
use cdbc::utils::ustr::UStr;
use crate::{PgTypeInfo, Postgres};

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct PgColumn {
    pub(crate) ordinal: usize,
    pub(crate) name: UStr,
    pub(crate) type_info: PgTypeInfo,
    #[serde(skip)]
    pub(crate) relation_id: Option<i32>,
    #[serde(skip)]
    pub(crate) relation_attribute_no: Option<i16>,
}

impl Column for PgColumn {
    type Database = Postgres;

    fn ordinal(&self) -> usize {
        self.ordinal
    }

    fn name(&self) -> &str {
        &*self.name
    }

    fn type_info(&self) -> &PgTypeInfo {
        &self.type_info
    }
}
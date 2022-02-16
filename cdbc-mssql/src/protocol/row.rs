use bytes::Bytes;
use cdbc::Error;

use cdbc::io::BufExt;
use crate::{MssqlColumn, MssqlTypeInfo};

#[derive(Debug)]
pub struct Row {
    pub column_types: Vec<MssqlTypeInfo>,
    pub values: Vec<Option<Bytes>>,
}

impl Row {
    pub fn get(
        buf: &mut Bytes,
        nullable: bool,
        columns: &[MssqlColumn],
    ) -> Result<Self, Error> {
        let mut values = Vec::with_capacity(columns.len());
        let mut column_types = Vec::with_capacity(columns.len());

        let nulls = if nullable {
            buf.get_bytes((columns.len() + 7) / 8)
        } else {
            Bytes::from_static(b"")
        };

        for (i, column) in columns.iter().enumerate() {
            column_types.push(column.type_info.clone());

            if !(column.type_info.0.is_null() || (nullable && (nulls[i / 8] & (1 << (i % 8))) != 0))
            {
                values.push(column.type_info.0.get_value(buf));
            } else {
                values.push(None);
            }
        }

        Ok(Self {
            values,
            column_types,
        })
    }
}

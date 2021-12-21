use bytes::{Buf, Bytes};

use cdbc::error::Error;
use cdbc::io::IoDecode;
use crate::io::MySqlBufExt;
use crate::protocol::Row;
use crate::MySqlColumn;

#[derive(Debug)]
pub(crate) struct TextRow(pub(crate) Row);

impl<'de> IoDecode<'de, &'de [MySqlColumn]> for TextRow {
    fn decode_with(mut buf: Bytes, columns: &'de [MySqlColumn]) -> Result<Self, Error> {
        let storage = buf.clone();
        let offset = buf.len();

        let mut values = Vec::with_capacity(columns.len());

        for _ in columns {
            if buf[0] == 0xfb {
                // NULL is sent as 0xfb
                values.push(None);
                buf.advance(1);
            } else {
                let size = buf.get_uint_lenenc() as usize;
                let offset = offset - buf.len();

                values.push(Some(offset..(offset + size)));

                buf.advance(size);
            }
        }

        Ok(TextRow(Row { values, storage }))
    }
}

use bytes::{Buf, Bytes};
use cdbc::Error;

use crate::io::MssqlBufExt;

#[derive(Debug)]
pub struct Info {
    pub number: u32,
    pub state: u8,
    pub class: u8,
    pub message: String,
    pub server: String,
    pub procedure: String,
    pub line: u32,
}

impl Info {
    pub fn get(buf: &mut Bytes) -> Result<Self, Error> {
        let len = buf.get_u16_le();
        let mut data = buf.split_to(len as usize);

        let number = data.get_u32_le();
        let state = data.get_u8();
        let class = data.get_u8();
        let message = data.get_us_varchar()?;
        let server = data.get_b_varchar()?;
        let procedure = data.get_b_varchar()?;
        let line = data.get_u32_le();

        Ok(Self {
            number,
            state,
            class,
            message,
            server,
            procedure,
            line,
        })
    }
}

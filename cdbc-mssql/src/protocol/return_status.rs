use bytes::{Buf, Bytes};
use cdbc::Error;


#[derive(Debug)]
pub struct ReturnStatus {
    value: i32,
}

impl ReturnStatus {
    pub fn get(buf: &mut Bytes) -> Result<Self, Error> {
        let value = buf.get_i32_le();

        Ok(Self { value })
    }
}

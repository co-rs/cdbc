use bytes::{Buf, Bytes};
use cdbc::Error;


#[derive(Debug)]
pub struct Order {
    columns: Bytes,
}

impl Order {
    pub fn get(buf: &mut Bytes) -> Result<Self, Error> {
        let len = buf.get_u16_le();
        let columns = buf.split_to(len as usize);

        Ok(Self { columns })
    }
}

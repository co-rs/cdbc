use bitflags::bitflags;
use bytes::{Buf, Bytes};
use cdbc::Error;

use crate::io::MssqlBufExt;
use crate::protocol::col_meta_data::Flags;
use crate::protocol::type_info::TypeInfo;

#[derive(Debug)]
pub struct ReturnValue {
    param_ordinal: u16,
    param_name: String,
    status: ReturnValueStatus,
    user_type: u32,
    flags: Flags,
    pub type_info: TypeInfo,
    pub value: Option<Bytes>,
}

bitflags! {
    pub struct ReturnValueStatus: u8 {
        // If ReturnValue corresponds to OUTPUT parameter of a stored procedure invocation
        const OUTPUT_PARAM = 0x01;

        // If ReturnValue corresponds to return value of User Defined Function.
        const USER_DEFINED = 0x02;
    }
}

impl ReturnValue {
    pub fn get(buf: &mut Bytes) -> Result<Self, Error> {
        let ordinal = buf.get_u16_le();
        let name = buf.get_b_varchar()?;
        let status = ReturnValueStatus::from_bits_truncate(buf.get_u8());
        let user_type = buf.get_u32_le();
        let flags = Flags::from_bits_truncate(buf.get_u16_le());
        let type_info = TypeInfo::get(buf)?;
        let value = type_info.get_value(buf);

        Ok(Self {
            param_ordinal: ordinal,
            param_name: name,
            status,
            user_type,
            flags,
            type_info,
            value,
        })
    }
}

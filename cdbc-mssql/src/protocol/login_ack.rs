use bytes::{Buf, Bytes};
use cdbc::Error;
use crate::io::MssqlBufExt;
use crate::protocol::pre_login::Version;

#[derive(Debug)]
pub struct LoginAck {
    pub interface: u8,
    pub tds_version: u32,
    pub program_name: String,
    pub program_version: Version,
}

impl LoginAck {
    pub fn get(buf: &mut Bytes) -> Result<Self, Error> {
        let len = buf.get_u16_le();
        let mut data = buf.split_to(len as usize);

        let interface = data.get_u8();
        let tds_version = data.get_u32_le();
        let program_name = data.get_b_varchar()?;
        let program_version_major = data.get_u8();
        let program_version_minor = data.get_u8();
        let program_version_build = data.get_u16();

        Ok(Self {
            interface,
            tds_version,
            program_name,
            program_version: Version {
                major: program_version_major,
                minor: program_version_minor,
                build: program_version_build,
                sub_build: 0,
            },
        })
    }
}

use byteorder::{ByteOrder, LittleEndian};

use cdbc::decode::Decode;
use cdbc::encode::{Encode, IsNull};
use cdbc::error::BoxDynError;
use crate::protocol::type_info::{DataType, TypeInfo};
use crate::{Mssql, MssqlTypeInfo, MssqlValueRef};
use cdbc::types::Type;

impl Type<Mssql> for f32 {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo(TypeInfo::new(DataType::FloatN, 4))
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        matches!(ty.0.ty, DataType::Real | DataType::FloatN) && ty.0.size == 4
    }
}

impl Encode<'_, Mssql> for f32 {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend(&self.to_le_bytes());

        IsNull::No
    }
}

impl Decode<'_, Mssql> for f32 {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        Ok(LittleEndian::read_f32(value.as_bytes()?))
    }
}

impl Type<Mssql> for f64 {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo(TypeInfo::new(DataType::FloatN, 8))
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        matches!(ty.0.ty, DataType::Float | DataType::FloatN) && ty.0.size == 8
    }
}

impl Encode<'_, Mssql> for f64 {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend(&self.to_le_bytes());

        IsNull::No
    }
}

impl Decode<'_, Mssql> for f64 {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        Ok(LittleEndian::read_f64(value.as_bytes()?))
    }
}

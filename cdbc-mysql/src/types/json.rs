use serde::{Deserialize, Serialize};

use cdbc::decode::Decode;
use cdbc::encode::{Encode, IsNull};
use cdbc::error::BoxDynError;
use crate::protocol::text::ColumnType;
use crate::{MySql, MySqlTypeInfo, MySqlValueRef};
use cdbc::types::{Json, Type};

impl<T> Type<MySql> for Json<T> {
    fn type_info() -> MySqlTypeInfo {
        // MySql uses the `CHAR` type to pass JSON data from and to the client
        // NOTE: This is forwards-compatible with MySQL v8+ as CHAR is a common transmission format
        //       and has nothing to do with the native storage ability of MySQL v8+
        MySqlTypeInfo::binary(ColumnType::String)
    }

    fn compatible(ty: &MySqlTypeInfo) -> bool {
        ty.r#type == ColumnType::Json
            || <&str as Type<MySql>>::compatible(ty)
            || <&[u8] as Type<MySql>>::compatible(ty)
    }
}

impl<T> Encode<'_, MySql> for Json<T>
where
    T: Serialize,
{
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        let json_string_value =
            serde_json::to_string(&self.0).expect("serde_json failed to convert to string");

        <&str as Encode<MySql>>::encode(json_string_value.as_str(), buf)
    }
}

impl<'r, T> Decode<'r, MySql> for Json<T>
where
    T: 'r + Deserialize<'r>,
{
    fn decode(value: MySqlValueRef<'r>) -> Result<Self, BoxDynError> {
        let string_value = <&str as Decode<MySql>>::decode(value)?;

        serde_json::from_str(&string_value)
            .map(Json)
            .map_err(Into::into)
    }
}

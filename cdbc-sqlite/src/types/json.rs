use serde::{Deserialize, Serialize};

use cdbc::decode::Decode;
use cdbc::encode::{Encode, IsNull};
use cdbc::error::BoxDynError;
use crate::{
    type_info::DataType, Sqlite, SqliteArgumentValue, SqliteTypeInfo, SqliteValueRef,
};
use cdbc::types::{Json, Type};

impl<T> Type<Sqlite> for Json<T> {
    fn type_info() -> SqliteTypeInfo {
        SqliteTypeInfo(DataType::Text)
    }

    fn compatible(ty: &SqliteTypeInfo) -> bool {
        <&str as Type<Sqlite>>::compatible(ty)
    }
}

impl<T> Encode<'_, Sqlite> for Json<T>
where
    T: Serialize,
{
    fn encode_by_ref(&self, buf: &mut Vec<SqliteArgumentValue<'_>>) -> IsNull {
        let json_string_value =
            serde_json::to_string(&self.0).expect("serde_json failed to convert to string");

        Encode::<Sqlite>::encode(json_string_value, buf)
    }
}

impl<'r, T> Decode<'r, Sqlite> for Json<T>
where
    T: 'r + Deserialize<'r>,
{
    fn decode(value: SqliteValueRef<'r>) -> Result<Self, BoxDynError> {
        let string_value = <&str as Decode<Sqlite>>::decode(value)?;

        serde_json::from_str(&string_value)
            .map(Json)
            .map_err(Into::into)
    }
}

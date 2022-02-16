use std::borrow::Cow;
use mco::std::time::time::{RFC3339Nano, Time};
use cdbc::decode::Decode;
use cdbc::error::BoxDynError;
use cdbc::types::Type;
use cdbc::encode::{Encode, IsNull};
use crate::{Sqlite, SqliteArgumentValue, SqliteTypeInfo, SqliteValueRef};

impl Type<Sqlite> for Time {
    fn type_info() -> SqliteTypeInfo {
        <&str as Type<Sqlite>>::type_info()
    }
}

impl<'q> Encode<'q, Sqlite> for Time {
    fn encode(self, args: &mut Vec<SqliteArgumentValue<'q>>) -> IsNull {
        args.push(SqliteArgumentValue::Text(Cow::Owned(self.to_string())));

        IsNull::No
    }

    fn encode_by_ref(&self, args: &mut Vec<SqliteArgumentValue<'q>>) -> IsNull {
        args.push(SqliteArgumentValue::Text(Cow::Owned(self.to_string())));

        IsNull::No
    }
}

impl<'r> Decode<'r, Sqlite> for Time {
    fn decode(value: SqliteValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(Time::parse(RFC3339Nano,value.text()?)?)
    }
}
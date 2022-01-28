use cdbc::decode::Decode;
use cdbc::encode::{Encode, IsNull};
use cdbc::error::BoxDynError;
use crate::types::time::PG_EPOCH;
use crate::{
    PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueFormat, PgValueRef, Postgres,
};
use cdbc::types::Type;
use std::mem;
use std::ops::Deref;
use time::{Date, Duration};

impl Type<Postgres> for Date {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::DATE
    }
}

impl PgHasArrayType for Date {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::DATE_ARRAY
    }
}

impl Encode<'_, Postgres> for Date {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        // DATE is encoded as the days since epoch
        let days = (*self - *PG_EPOCH.deref()).whole_days() as i32;
        Encode::<Postgres>::encode(&days, buf)
    }

    fn size_hint(&self) -> usize {
        mem::size_of::<i32>()
    }
}

impl<'r> Decode<'r, Postgres> for Date {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(match value.format() {
            PgValueFormat::Binary => {
                // DATE is encoded as the days since epoch
                let days: i32 = Decode::<Postgres>::decode(value)?;
                *PG_EPOCH.deref() + Duration::days(days.into())
            }

            PgValueFormat::Text => {
                let format = time::format_description::parse("[year]-[month]-[day]")?;
                Date::parse(value.as_str()?, &format)?
            },
        })
    }
}

use cdbc::decode::Decode;
use cdbc::encode::{Encode, IsNull};
use cdbc::error::BoxDynError;
use crate::{
    PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueFormat, PgValueRef, Postgres,
};
use cdbc::types::Type;
use std::borrow::Cow;
use std::mem;
use time::{Duration, Time};

impl Type<Postgres> for Time {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::TIME
    }
}

impl PgHasArrayType for Time {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::TIME_ARRAY
    }
}

impl Encode<'_, Postgres> for Time {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        // TIME is encoded as the microseconds since midnight
        let us = (*self - Time::from_hms(0,0,0).unwrap()).whole_microseconds() as i64;
        Encode::<Postgres>::encode(&us, buf)
    }

    fn size_hint(&self) -> usize {
        mem::size_of::<u64>()
    }
}

impl<'r> Decode<'r, Postgres> for Time {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(match value.format() {
            PgValueFormat::Binary => {
                // TIME is encoded as the microseconds since midnight
                let us = Decode::<Postgres>::decode(value)?;
                Time::from_hms(0,0,0).unwrap() + Duration::microseconds(us)
            }

            PgValueFormat::Text => {
                // If there are less than 9 digits after the decimal point
                // We need to zero-pad

                // FIXME: Ask [time] to add a parse % for less-than-fixed-9 nanos

                let s = value.as_str()?;

                let s = if s.len() < 20 {
                    Cow::Owned(format!("{:0<19}", s))
                } else {
                    Cow::Borrowed(s)
                };
                let format = time::format_description::parse("[hour]:[minute]:[second].[subsecond]")?;
                Time::parse(&*s as &str, &format)?
            }
        })
    }
}

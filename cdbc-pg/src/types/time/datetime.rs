use cdbc::decode::Decode;
use cdbc::encode::{Encode, IsNull};
use cdbc::error::BoxDynError;
use crate::types::time::PG_EPOCH;
use crate::{
    PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueFormat, PgValueRef, Postgres,
};
use cdbc::types::Type;
use std::borrow::Cow;
use std::mem;

use time::{Duration, OffsetDateTime, PrimitiveDateTime, UtcOffset};


impl Type<Postgres> for PrimitiveDateTime {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::TIMESTAMP
    }
}

impl Type<Postgres> for OffsetDateTime {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::TIMESTAMPTZ
    }
}

impl PgHasArrayType for PrimitiveDateTime {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::TIMESTAMP_ARRAY
    }
}

impl PgHasArrayType for OffsetDateTime {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::TIMESTAMPTZ_ARRAY
    }
}

impl Encode<'_, Postgres> for PrimitiveDateTime {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        // TIMESTAMP is encoded as the microseconds since the epoch
        let us = (*self - PG_EPOCH.midnight()).whole_microseconds() as i64;
        Encode::<Postgres>::encode(&us, buf)
    }

    fn size_hint(&self) -> usize {
        mem::size_of::<i64>()
    }
}

impl<'r> Decode<'r, Postgres> for PrimitiveDateTime {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(match value.format() {
            PgValueFormat::Binary => {
                // TIMESTAMP is encoded as the microseconds since the epoch
                let us = Decode::<Postgres>::decode(value)?;
                PG_EPOCH.midnight() + Duration::microseconds(us)
            }

            PgValueFormat::Text => {
                // If there are less than 9 digits after the decimal point
                // We need to zero-pad

                // TODO: De-duplicate with MySQL
                // TODO: Ask [time] to add a parse % for less-than-fixed-9 nanos

                let s = value.as_str()?;

                let s = if let Some(plus) = s.rfind('+') {
                    let mut big = String::from(&s[..plus]);

                    while big.len() < 31 {
                        big.push('0');
                    }

                    big.push_str(&s[plus..]);

                    Cow::Owned(big)
                } else if s.len() < 31 {
                    if s.contains('.') {
                        Cow::Owned(format!("{:0<30}", s))
                    } else {
                        Cow::Owned(format!("{}.000000000", s))
                    }
                } else {
                    Cow::Borrowed(s)
                };
                let format = time::format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond]")?;
                PrimitiveDateTime::parse(&*s, &format)?
            }
        })
    }
}

impl Encode<'_, Postgres> for OffsetDateTime {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        let utc = self.to_offset(UtcOffset::UTC);
        let primitive = PrimitiveDateTime::new(utc.date(), utc.time());

        Encode::<Postgres>::encode(&primitive, buf)
    }

    fn size_hint(&self) -> usize {
        mem::size_of::<i64>()
    }
}

impl<'r> Decode<'r, Postgres> for OffsetDateTime {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(<PrimitiveDateTime as Decode<Postgres>>::decode(value)?.assume_utc())
    }
}


impl Type<Postgres> for cogo::std::time::time::Time{
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::TIMESTAMPTZ
    }
}

impl Encode<'_, Postgres> for cogo::std::time::time::Time {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        self.inner.encode_by_ref(buf)
    }
}

impl<'r> Decode<'r, Postgres> for cogo::std::time::time::Time {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(cogo::std::time::time::Time{ inner: OffsetDateTime::decode(value)? })
    }
}
use bytes::Bytes;

use crate::error::Error;

pub trait IoDecode<'de, Context = ()>
where
    Self: Sized,
{
    fn decode(buf: Bytes) -> Result<Self, Error>
    where
        Self: IoDecode<'de, ()>,
    {
        Self::decode_with(buf, ())
    }

    fn decode_with(buf: Bytes, context: Context) -> Result<Self, Error>;
}

impl IoDecode<'_> for Bytes {
    fn decode_with(buf: Bytes, _: ()) -> Result<Self, Error> {
        Ok(buf)
    }
}

impl IoDecode<'_> for () {
    fn decode_with(_: Bytes, _: ()) -> Result<(), Error> {
        Ok(())
    }
}

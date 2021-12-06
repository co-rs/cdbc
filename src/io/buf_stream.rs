use bytes::BytesMut;
use std::io::Cursor;

pub struct BufStream<S>
    where
        S: Unpin,
{
    pub(crate) stream: S,

    // writes with `write` to the underlying stream are buffered
    // this can be flushed with `flush`
    pub(crate) wbuf: Vec<u8>,

    // we read into the read buffer using 100% safe code
    rbuf: BytesMut,
}


impl<S> BufStream<S> where
    S: Unpin,
{
    pub fn new(stream: S) -> Self {
        Self {
            stream,
            wbuf: Vec::with_capacity(512),
            rbuf: BytesMut::with_capacity(4096),
        }
    }
    // pub fn write<'en, T>(&mut self, value: T)
    //     where
    //         T: Encode<'en, ()>,
    // {
    //     self.write_with(value, ())
    // }
    // pub fn write_with<'en, T, C>(&mut self, value: T, context: C)
    //     where
    //         T: Encode<'en, C>,
    // {
    //     value.encode_with(&mut self.wbuf, context);
    // }
    // pub fn flush(&mut self) -> WriteAndFlush<'_, S> {
    //     WriteAndFlush {
    //         stream: &mut self.stream,
    //         buf: Cursor::new(&mut self.wbuf),
    //     }
    // }


}
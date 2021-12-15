use bytes::BytesMut;
use std::io::Cursor;
use may::net::TcpStream;
use crate::io::encode::Encode;
use crate::io::write_and_flush::WriteAndFlush;

pub struct BufStream {
    pub(crate) stream: TcpStream,

    // writes with `write` to the underlying stream are buffered
    // this can be flushed with `flush`
    pub(crate) wbuf: Vec<u8>,

    // we read into the read buffer using 100% safe code
    rbuf: BytesMut,
}


impl BufStream where
{
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            wbuf: Vec::with_capacity(512),
            rbuf: BytesMut::with_capacity(4096),
        }
    }
    pub fn write<'en, T>(&mut self, value: T)
        where
            T: Encode<'en, ()>,
    {
        self.write_with(value, ())
    }
    pub fn write_with<'en, T, C>(&mut self, value: T, context: C)
        where
            T: Encode<'en, C>,
    {
        value.encode_with(&mut self.wbuf, context);
    }
    pub fn flush(&mut self) -> WriteAndFlush<'_> {
        WriteAndFlush {
            stream: &mut self.stream,
            buf: Cursor::new(&mut self.wbuf),
        }
    }
}
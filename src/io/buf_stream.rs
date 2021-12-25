use bytes::BytesMut;
use std::io::{Cursor, Read, Write};
use cogo::net::TcpStream;
use crate::Error;
use crate::io::Decode;
use crate::io::encode::Encode;
use crate::io::write_and_flush::WriteAndFlush;
use std::io;
use std::ops::{Deref, DerefMut};

pub struct BufStream<T> where T: Write + Read {
    pub stream: T,

    // writes with `write` to the underlying stream are buffered
    // this can be flushed with `flush`
    pub wbuf: Vec<u8>,

    // we read into the read buffer using 100% safe code
    rbuf: BytesMut,
}


impl<T> BufStream<T> where T: Write + Read
{
    pub fn new(stream: T) -> Self {
        Self {
            stream,
            wbuf: Vec::with_capacity(512),
            rbuf: BytesMut::with_capacity(4096),
        }
    }
    pub fn write<'en,A>(&mut self, value: A)
        where
            A: Encode<'en, ()>,
    {
        self.write_with(value, ())
    }

    pub fn write_with<'en, C,A>(&mut self, value: A, context: C)
        where
            A: Encode<'en, C>,
    {
        value.encode_with(&mut self.wbuf, context);
    }
    pub fn read<'de,A>(&mut self, cnt: usize) -> Result<A, Error>
        where
            A: Decode<'de, ()>,
    {
        self.read_with(cnt, ())
    }

    pub fn read_with<'de, C,A>(&mut self, cnt: usize, context: C) -> Result<A, Error>
        where
            A: Decode<'de, C>,
    {
        A::decode_with(self.read_raw(cnt)?.freeze(), context)
    }

    pub fn read_raw(&mut self, cnt: usize) -> Result<BytesMut, Error> {
        read_raw_into(&mut self.stream, &mut self.rbuf, cnt)?;
        let buf = self.rbuf.split_to(cnt);
        Ok(buf)
    }

    pub fn read_raw_into(&mut self, buf: &mut BytesMut, cnt: usize) -> Result<(), Error> {
        read_raw_into(&mut self.stream, buf, cnt)
    }

    pub fn flush(&mut self) -> crate::error::Result<()> {
        WriteAndFlush {
            stream: &mut self.stream,
            buf: Cursor::new(&mut self.wbuf),
        }.flush()
    }
}

fn read_raw_into<S: Read + Write>(
    stream: &mut S,
    buf: &mut BytesMut,
    cnt: usize,
) -> Result<(), Error> {
    let mut buf = BufTruncator::new(buf);
    buf.reserve(cnt);

    while !buf.is_full() {
        let n = buf.read(stream)?;

        if n == 0 {
            // a zero read when we had space in the read buffer
            // should be treated as an EOF

            // and an unexpected EOF means the server told us to go away

            return Err(io::Error::from(io::ErrorKind::ConnectionAborted).into());
        }
    }

    Ok(())
}

// Holds a buffer which has been temporarily extended, so that
// we can read into it. Automatically shrinks the buffer back
// down if the read is cancelled.
struct BufTruncator<'a> {
    buf: &'a mut BytesMut,
    filled_len: usize,
}

impl<'a> BufTruncator<'a> {
    fn new(buf: &'a mut BytesMut) -> Self {
        let filled_len = buf.len();
        Self { buf, filled_len }
    }
    fn reserve(&mut self, space: usize) {
        self.buf.resize(self.filled_len + space, 0);
    }
    fn read<S: Read>(&mut self, stream: &mut S) -> Result<usize, Error> {
        let n = stream.read(&mut self.buf[self.filled_len..])?;
        self.filled_len += n;
        Ok(n)
    }
    fn is_full(&self) -> bool {
        self.filled_len >= self.buf.len()
    }
}

impl Drop for BufTruncator<'_> {
    fn drop(&mut self) {
        self.buf.truncate(self.filled_len);
    }
}

impl <T>Deref for BufStream<T>where T: Write + Read{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.stream
    }
}
impl <T>DerefMut for BufStream<T> where T: Write + Read{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.stream
    }
}

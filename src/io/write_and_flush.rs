use std::io::{BufRead, Cursor, Read, Write};
use crate::error::Error;
use mco::net::TcpStream;

/// Atomic operation that writes the full buffer to the stream, flushes the stream, and then
/// clears the buffer (even if either of the two previous operations failed).
pub struct WriteAndFlush<'a,T>where T:Write+Read {
    pub(super) stream: &'a mut T,
    pub(super) buf: Cursor<&'a mut Vec<u8>>,
}

impl <'a,T>WriteAndFlush<'a,T> where T:Write+Read{
    pub fn flush(mut self)->crate::error::Result<()>{
        loop {
            let read = self.buf.fill_buf()?;
            if !read.is_empty() {
                let written = self.stream.write(read)?;
                self.buf.consume(written);
            } else {
                break;
            }
        }
        self.stream.flush()?;
        return Ok(());
    }
}

impl<'a,T> Drop for WriteAndFlush<'a,T> where T:Write+Read{
    fn drop(&mut self) {
        // clear the buffer regardless of whether the flush succeeded or not
        self.buf.get_mut().clear();
    }
}



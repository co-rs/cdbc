use std::io::{BufRead, Cursor, Write};
use crate::error::Error;
use may::net::TcpStream;

/// Atomic operation that writes the full buffer to the stream, flushes the stream, and then
/// clears the buffer (even if either of the two previous operations failed).
pub struct WriteAndFlush<'a> {
    pub(super) stream: &'a mut TcpStream,
    pub(super) buf: Cursor<&'a mut Vec<u8>>,
}

impl <'a>WriteAndFlush<'a> {
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

impl<'a> Drop for WriteAndFlush<'a> {
    fn drop(&mut self) {
        // clear the buffer regardless of whether the flush succeeded or not
        self.buf.get_mut().clear();
    }
}



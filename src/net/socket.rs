#![allow(dead_code)]

use std::io;
use std::io::{Read, Write};
use std::net::Shutdown;
use std::path::Path;

use std::task::{Context, Poll};
use may::net::TcpStream;


#[derive(Debug)]
pub struct Socket {
    pub inner: TcpStream,
}

impl Socket {

    pub fn connect_tcp(host: &str, port: u16) -> io::Result<Self> {
       Ok(Self{
           inner: TcpStream::connect((host, port))?
       })
    }

    pub fn shutdown(&mut self) -> io::Result<()> {
        {
            self.inner.shutdown(Shutdown::Both)
        }
    }
}

impl std::io::Read for Socket {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>{
        self.inner.read(buf)
    }
}

impl std::io::Write for Socket {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}


pub trait IsTLS{
    #[inline]
    fn is_tls(&self) -> bool;
}

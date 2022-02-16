#![allow(dead_code)]

use std::io;
use std::io::{ErrorKind, Read, Write};
use std::net::{Shutdown, ToSocketAddrs};
use std::path::Path;

use std::task::{Context, Poll};
use std::time::Duration;
use mco::net::TcpStream;


#[derive(Debug)]
pub struct Socket {
    pub inner: TcpStream,
}

impl Socket {
    pub fn connect_tcp_timeout(host: &str, port: u16, d: Duration) -> io::Result<Self> {
        let mut addrs = (host, port).to_socket_addrs()?;
        let next = addrs.next();
        if next.is_none() {
            return Err(io::Error::new(ErrorKind::NotFound, "addr not find"));
        }
        Ok(Self {
            inner: TcpStream::connect_timeout(&next.unwrap(), d)?
        })
    }

    pub fn connect_tcp(host: &str, port: u16) -> io::Result<Self> {
        Ok(Self {
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
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
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


pub trait IsTLS {
    #[inline]
    fn is_tls(&self) -> bool;
}

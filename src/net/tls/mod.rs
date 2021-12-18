#![allow(dead_code)]

use std::io;
use std::io::{Read, Write};
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::error::Error;
use std::mem::replace;
use native_tls::HandshakeError;
use crate::decode::Decode;
use crate::encode::Encode;
use crate::net::socket::IsTLS;


/// X.509 Certificate input, either a file path or a PEM encoded inline certificate(s).
#[derive(Clone, Debug)]
pub enum CertificateInput {
    /// PEM encoded certificate(s)
    Inline(Vec<u8>),
    /// Path to a file containing PEM encoded certificate(s)
    File(PathBuf),
}

impl From<String> for CertificateInput {
    fn from(value: String) -> Self {
        let trimmed = value.trim();
        // Some heuristics according to https://tools.ietf.org/html/rfc7468
        if trimmed.starts_with("-----BEGIN CERTIFICATE-----")
            && trimmed.contains("-----END CERTIFICATE-----")
        {
            CertificateInput::Inline(value.as_bytes().to_vec())
        } else {
            CertificateInput::File(PathBuf::from(value))
        }
    }
}

impl CertificateInput {
    fn data(&self) -> Result<Vec<u8>, std::io::Error> {
        match self {
            CertificateInput::Inline(v) => Ok(v.clone()),
            CertificateInput::File(path) => std::fs::read(path),
        }
    }
}

impl std::fmt::Display for CertificateInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CertificateInput::Inline(v) => write!(f, "{}", String::from_utf8_lossy(v.as_slice())),
            CertificateInput::File(path) => write!(f, "file: {}", path.display()),
        }
    }
}


pub struct TlsStream<S>{
    pub inner:native_tls::TlsStream<S>
}
impl <S>Deref for TlsStream<S>{
    type Target = native_tls::TlsStream<S>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl <S>DerefMut for TlsStream<S>{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}



pub enum MaybeTlsStream<S>
where
    S: std::io::Read + std::io::Write,
{
    Raw(S),
    Tls(TlsStream<S>),
    Upgrading,
}


impl<S> MaybeTlsStream<S>
where
    S: std::io::Read + std::io::Write  + std::fmt::Debug + Send +Sync + 'static,
{
    pub fn upgrade(
        &mut self,
        host: &str,
        accept_invalid_certs: bool,
        accept_invalid_hostnames: bool,
        root_cert_path: Option<&CertificateInput>,
    ) -> crate::Result<()> {
        let connector = configure_tls_connector(
            accept_invalid_certs,
            accept_invalid_hostnames,
            root_cert_path,
        )?;

        let stream = match replace(self, MaybeTlsStream::Upgrading) {
            MaybeTlsStream::Raw(stream) => stream,

            MaybeTlsStream::Tls(_) => {
                // ignore upgrade, we are already a TLS connection
                return Ok(());
            }

            MaybeTlsStream::Upgrading => {
                // we previously failed to upgrade and now hold no connection
                // this should only happen from an internal misuse of this method
                return Err(Error::Io(io::ErrorKind::ConnectionAborted.into()));
            }
        };

        let s = connector.connect(host, stream);
        match s{
            Ok(s) => {
                *self = MaybeTlsStream::Tls(TlsStream{
                    inner: s
                });
            }
            Err(e) => {
               return Err(Error::Tls(Box::new(e)));
            }
        }
        Ok(())
    }
}

impl <S>IsTLS for MaybeTlsStream<S> where S:Write+Read{
    #[inline]
    fn is_tls(&self) -> bool {
        matches!(self, Self::Tls(_))
    }
}

// #[cfg(feature = "_tls-native-tls")]
fn configure_tls_connector(
    accept_invalid_certs: bool,
    accept_invalid_hostnames: bool,
    root_cert_path: Option<&CertificateInput>,
) -> Result<native_tls::TlsConnector, Error> {
    use native_tls::{Certificate, TlsConnector};

    let mut builder = TlsConnector::builder();
    builder
        .danger_accept_invalid_certs(accept_invalid_certs)
        .danger_accept_invalid_hostnames(accept_invalid_hostnames);

    if !accept_invalid_certs {
        if let Some(ca) = root_cert_path {
            let data = ca.data()?;
            let cert = {
                match Certificate::from_pem(&data){
                    Ok(v) => {v}
                    Err(e) => {
                        return Err(Error::Tls(Box::new(e)));
                    }
                }
            };
            builder.add_root_certificate(cert);
        }
    }

    let connector = match builder.build(){
        Ok(v) => {v}
        Err(e) => {
            return Err(Error::Tls(Box::new(e)));
        }
    };
    Ok(connector)
}



impl<S> Read for MaybeTlsStream<S>
where
    S: Unpin + Write + Read,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match &mut *self {
            MaybeTlsStream::Raw(s) => s.read(buf),
            MaybeTlsStream::Tls(s) => s.read(buf),
            MaybeTlsStream::Upgrading => Err(io::ErrorKind::ConnectionAborted.into()),
        }
    }
}

impl<S> std::io::Write for MaybeTlsStream<S>
where
    S: Unpin + Write + Read,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            MaybeTlsStream::Raw(s) => s.write( buf),
            MaybeTlsStream::Tls(s) => s.write( buf),

            MaybeTlsStream::Upgrading => Err(io::ErrorKind::ConnectionAborted.into()),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            MaybeTlsStream::Raw(s) => s.flush(),
            MaybeTlsStream::Tls(s) => s.flush(),

            MaybeTlsStream::Upgrading => Err(io::ErrorKind::ConnectionAborted.into()),
        }
    }
    
}

impl<S> Deref for MaybeTlsStream<S>
where
    S: Unpin + Write + Read,
{
    type Target = S;

    fn deref(&self) -> &Self::Target {
        match self {
            MaybeTlsStream::Raw(s) => s,

            MaybeTlsStream::Tls(s) => s.get_ref(),

            MaybeTlsStream::Upgrading => {
                panic!("{}", io::Error::from(io::ErrorKind::ConnectionAborted))
            }
        }
    }
}

impl<S> DerefMut for MaybeTlsStream<S>
where
    S: Unpin + Write + Read,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            MaybeTlsStream::Raw(s) => s,

            MaybeTlsStream::Tls(s) => s.get_mut(),

            MaybeTlsStream::Upgrading => {
                panic!("{}", io::Error::from(io::ErrorKind::ConnectionAborted))
            }
        }
    }
}

// impl<S>  MaybeTlsStream<S>
//     where
//         S: Unpin + Write + Read,
// {
//     pub fn write_with<'en, C>(&mut self, value: T, context: C)
//         where
//             T: Encode<'en, C>{
//
//     }
//     pub fn read_with<'de, C>(&mut self, cnt: usize, context: C) -> Result<T, Error>
//         where
//             T: Decode<'de, C>{
//
//     }
//
// }

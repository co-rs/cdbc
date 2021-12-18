mod socket;
mod tls;

pub use socket::Socket;
pub use tls::{CertificateInput, MaybeTlsStream};

// #[cfg(feature = "_rt-async-std")]
type PollReadBuf<'a> = Vec<u8>;
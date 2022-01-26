use std::collections::BTreeMap;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

use bytes::{Buf, Bytes};
use cdbc::error::Error;
use cdbc::io::{BufStream, Decode, Encode};
use cdbc::net::{MaybeTlsStream, Socket};
use crate::message::{Message, MessageFormat, Notice, Notification, ParameterStatus};
use crate::{PgConnectOptions, PgDatabaseError, PgSeverity};
use cogo::std::sync::channel::{Receiver, Sender};

// the stream is a separate type from the connection to uphold the invariant where an instantiated
// [PgConnection] is a **valid** connection to postgres

// when a new connection is asked for, we work directly on the [PgStream] type until the
// connection is fully established

// in other words, `self` in any PgConnection method is a live connection to postgres that
// is fully prepared to receive queries

pub struct PgStream {
    inner: BufStream<MaybeTlsStream<Socket>>,

    // buffer of unreceived notification messages from `PUBLISH`
    // this is set when creating a PgListener and only written to if that listener is
    // re-used for query execution in-between receiving messages
    pub(crate) notifications: Option<Sender<Notification>>,

    pub(crate) parameter_statuses: BTreeMap<String, String>,

    pub(crate) server_version_num: Option<u32>,
}

impl PgStream {
    pub fn connect(options: &PgConnectOptions, d: std::time::Duration) -> Result<Self, Error> {
        let socket = Socket::connect_tcp_timeout(&options.host, options.port, d)?;

        let inner = BufStream::new(MaybeTlsStream::Raw(socket));

        Ok(Self {
            inner,
            notifications: None,
            parameter_statuses: BTreeMap::default(),
            server_version_num: None,
        })
    }

    pub(crate) fn send<'en, T>(&mut self, message: T) -> Result<(), Error>
        where
            T: Encode<'en>,
    {
        self.write(message);
        self.flush()
    }

    // Expect a specific type and format
    pub(crate) fn recv_expect<'de, T: Decode<'de>>(
        &mut self,
        format: MessageFormat,
    ) -> Result<T, Error> {
        let message = self.recv()?;

        if message.format != format {
            return Err(err_protocol!(
                "expecting {:?} but received {:?}",
                format,
                message.format
            ));
        }

        message.decode()
    }

    pub(crate) fn recv_unchecked(&mut self) -> Result<Message, Error> {
        // all packets in postgres start with a 5-byte header
        // this header contains the message type and the total length of the message
        let mut header: Bytes = self.inner.read(5)?;

        let format = MessageFormat::try_from_u8(header.get_u8())?;
        let size = (header.get_u32() - 4) as usize;

        let contents = self.inner.read(size)?;

        Ok(Message { format, contents })
    }

    // Get the next message from the server
    // May wait for more data from the server
    pub(crate) fn recv(&mut self) -> Result<Message, Error> {
        loop {
            let message = self.recv_unchecked()?;

            match message.format {
                MessageFormat::ErrorResponse => {
                    // An error returned from the database server.
                    return Err(PgDatabaseError(message.decode()?).into());
                }

                MessageFormat::NotificationResponse => {
                    if let Some(buffer) = &mut self.notifications {
                        let notification: Notification = message.decode()?;
                        let _ = buffer.send(notification);

                        continue;
                    }
                }

                MessageFormat::ParameterStatus => {
                    // informs the frontend about the current (initial)
                    // setting of backend parameters

                    let ParameterStatus { name, value } = message.decode()?;
                    // TODO: handle `client_encoding`, `DateStyle` change

                    match name.as_str() {
                        "server_version" => {
                            self.server_version_num = parse_server_version(&value);
                        }
                        _ => {
                            self.parameter_statuses.insert(name, value);
                        }
                    }

                    continue;
                }

                MessageFormat::NoticeResponse => {
                    // do we need this to be more configurable?
                    // if you are reading this comment and think so, open an issue

                    //let notice: Notice = message.decode()?;

                    continue;
                }

                _ => {}
            }

            return Ok(message);
        }
    }
}

impl Deref for PgStream {
    type Target = BufStream<MaybeTlsStream<Socket>>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for PgStream {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

// reference:
// https://github.com/postgres/postgres/blob/6feebcb6b44631c3dc435e971bd80c2dd218a5ab/src/interfaces/libpq/fe-exec.c#L1030-L1065
fn parse_server_version(s: &str) -> Option<u32> {
    let mut parts = Vec::<u32>::with_capacity(3);

    let mut from = 0;
    let mut chs = s.char_indices().peekable();
    while let Some((i, ch)) = chs.next() {
        match ch {
            '.' => {
                if let Ok(num) = u32::from_str(&s[from..i]) {
                    parts.push(num);
                    from = i + 1;
                } else {
                    break;
                }
            }
            _ if ch.is_digit(10) => {
                if chs.peek().is_none() {
                    if let Ok(num) = u32::from_str(&s[from..]) {
                        parts.push(num);
                    }
                    break;
                }
            }
            _ => {
                if let Ok(num) = u32::from_str(&s[from..i]) {
                    parts.push(num);
                }
                break;
            }
        };
    }

    let version_num = match parts.as_slice() {
        [major, minor, rev] => (100 * major + minor) * 100 + rev,
        [major, minor] if *major >= 10 => 100 * 100 * major + minor,
        [major, minor] => (100 * major + minor) * 100,
        [major] => 100 * 100 * major,
        _ => return None,
    };

    Some(version_num)
}

#[cfg(test)]
mod tests {
    use super::parse_server_version;

    #[test]
    fn test_parse_server_version_num() {
        // old style
        assert_eq!(parse_server_version("9.6.1"), Some(90601));
        // new style
        assert_eq!(parse_server_version("10.1"), Some(100001));
        // old style without minor version
        assert_eq!(parse_server_version("9.6devel"), Some(90600));
        // new style without minor version, e.g.  */
        assert_eq!(parse_server_version("10devel"), Some(100000));
        assert_eq!(parse_server_version("13devel87"), Some(130000));
        // unknown
        assert_eq!(parse_server_version("unknown"), None);
    }
}

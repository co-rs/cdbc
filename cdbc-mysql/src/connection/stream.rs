use std::collections::VecDeque;
use std::io;
use std::ops::{Deref, DerefMut};

use bytes::{Buf, Bytes};

use cdbc::error::Error;
use cdbc::io::{BufStream, IoDecode, IoEncode};
use crate::collation::{CharSet, Collation};
use crate::io::MySqlBufExt;
use crate::protocol::response::{EofPacket, ErrPacket, OkPacket, Status};
use crate::protocol::{Capabilities, Packet};
use crate::{MySqlConnectOptions, MySqlDatabaseError};
use cdbc::net::{IsTLS, MaybeTlsStream, Socket};

pub struct MySqlStream {
    stream: BufStream<MaybeTlsStream<Socket>>,
    pub(crate) server_version: (u16, u16, u16),
    pub(super) capabilities: Capabilities,
    pub(crate) sequence_id: u8,
    pub(crate) waiting: VecDeque<Waiting>,
    pub(crate) charset: CharSet,
    pub(crate) collation: Collation,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Waiting {
    // waiting for a result set
    Result,

    // waiting for a row within a result set
    Row,
}


impl IsTLS for MySqlStream{
    #[inline]
    fn is_tls(&self) -> bool{
        self.stream.is_tls()
    }
}

impl MySqlStream {
    pub(super) fn connect(options: &MySqlConnectOptions) -> Result<Self, Error> {
        let charset: CharSet = options.charset.parse()?;
        let collation: Collation = options
            .collation
            .as_deref()
            .map(|collation| collation.parse())
            .transpose()?
            .unwrap_or_else(|| charset.default_collation());

        // let socket = match options.socket {
        //     Some(ref path) => Socket::connect_uds(path)?,
        //     None => Socket::connect_tcp(&options.host, options.port)?,
        // };
        let socket = Socket::connect_tcp(&options.host, options.port)?;

        let mut capabilities = Capabilities::PROTOCOL_41
            | Capabilities::IGNORE_SPACE
            | Capabilities::DEPRECATE_EOF
            | Capabilities::FOUND_ROWS
            | Capabilities::TRANSACTIONS
            | Capabilities::SECURE_CONNECTION
            | Capabilities::PLUGIN_AUTH_LENENC_DATA
            | Capabilities::MULTI_STATEMENTS
            | Capabilities::MULTI_RESULTS
            | Capabilities::PLUGIN_AUTH
            | Capabilities::PS_MULTI_RESULTS
            | Capabilities::SSL;

        if options.database.is_some() {
            capabilities |= Capabilities::CONNECT_WITH_DB;
        }

        Ok(Self {
            waiting: VecDeque::new(),
            capabilities,
            server_version: (0, 0, 0),
            sequence_id: 0,
            collation,
            charset,
            stream: BufStream::new(MaybeTlsStream::Raw(socket)),
        })
    }

    pub(crate) fn wait_until_ready(&mut self) -> Result<(), Error> {
        if !self.stream.wbuf.is_empty() {
            self.stream.flush()?;
        }

        while !self.waiting.is_empty() {
            while self.waiting.front() == Some(&Waiting::Row) {
                let packet = self.recv_packet()?;

                if packet[0] == 0xfe && packet.len() < 9 {
                    let eof = packet.eof(self.capabilities)?;

                    if eof.status.contains(Status::SERVER_MORE_RESULTS_EXISTS) {
                        *self.waiting.front_mut().unwrap() = Waiting::Result;
                    } else {
                        self.waiting.pop_front();
                    };
                }
            }

            while self.waiting.front() == Some(&Waiting::Result) {
                let packet = self.recv_packet()?;

                if packet[0] == 0x00 || packet[0] == 0xff {
                    let ok = packet.ok()?;

                    if !ok.status.contains(Status::SERVER_MORE_RESULTS_EXISTS) {
                        self.waiting.pop_front();
                    }
                } else {
                    *self.waiting.front_mut().unwrap() = Waiting::Row;
                    self.skip_result_metadata(packet)?;
                }
            }
        }

        Ok(())
    }

    pub(crate) fn send_packet<'en, T>(&mut self, payload: T) -> Result<(), Error>
    where
        T: IoEncode<'en, Capabilities>,
    {
        self.sequence_id = 0;
        self.write_packet(payload);
        self.flush()
    }

    pub(crate) fn write_packet<'en, T>(&mut self, payload: T)
    where
        T: IoEncode<'en, Capabilities>,
    {
        self.stream
            .write_with(Packet(payload), (self.capabilities, &mut self.sequence_id));
    }

    // receive the next packet from the database server
    // may block (async) on more data from the server
    pub(crate) fn recv_packet(&mut self) -> Result<Packet<Bytes>, Error> {
        // https://dev.mysql.com/doc/dev/mysql-server/8.0.12/page_protocol_basic_packets.html
        // https://mariadb.com/kb/en/library/0-packet/#standard-packet

        let mut header: Bytes = self.stream.read(4)?;

        let packet_size = header.get_uint_le(3) as usize;
        let sequence_id = header.get_u8();

        self.sequence_id = sequence_id.wrapping_add(1);

        let payload: Bytes = self.stream.read(packet_size)?;

        // TODO: packet compression
        // TODO: packet joining

        if payload[0] == 0xff {
            self.waiting.pop_front();

            // instead of letting this packet be looked at everywhere, we check here
            // and emit a proper Error
            return Err(
                MySqlDatabaseError(ErrPacket::decode_with(payload, self.capabilities)?).into(),
            );
        }

        Ok(Packet(payload))
    }

    pub(crate) fn recv<'de, T>(&mut self) -> Result<T, Error>
    where
        T: IoDecode<'de, Capabilities>,
    {
        self.recv_packet()?.decode_with(self.capabilities)
    }

    pub(crate) fn recv_ok(&mut self) -> Result<OkPacket, Error> {
        self.recv_packet()?.ok()
    }

    pub(crate) fn maybe_recv_eof(&mut self) -> Result<Option<EofPacket>, Error> {
        if self.capabilities.contains(Capabilities::DEPRECATE_EOF) {
            Ok(None)
        } else {
            self.recv().map(Some)
        }
    }

    fn skip_result_metadata(&mut self, mut packet: Packet<Bytes>) -> Result<(), Error> {
        let num_columns: u64 = packet.get_uint_lenenc(); // column count

        for _ in 0..num_columns {
            let _ = self.recv_packet()?;
        }

        self.maybe_recv_eof()?;

        Ok(())
    }

    pub fn shutdown(&mut self) ->io::Result<()>{
        self.stream.shutdown()
    }
}

impl Deref for MySqlStream {
    type Target = BufStream<MaybeTlsStream<Socket>>;

    fn deref(&self) -> &Self::Target {
        &self.stream
    }
}

impl DerefMut for MySqlStream {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.stream
    }
}

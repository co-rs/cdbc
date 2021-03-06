use std::time::Duration;
use bytes::buf::Buf;
use bytes::Bytes;

use cdbc::utils::statement_cache::StatementCache;
use cdbc::error::Error;
use crate::connection::{tls, MySqlStream, MAX_PACKET_SIZE};
use crate::protocol::connect::{
    AuthSwitchRequest, AuthSwitchResponse, Handshake, HandshakeResponse,
};
use crate::protocol::Capabilities;
use crate::{MySqlConnectOptions, MySqlConnection, MySqlSslMode};

impl MySqlConnection {
    pub(crate) fn establish(options: &MySqlConnectOptions, d: Duration) -> Result<Self, Error> {
        let mut stream: MySqlStream = MySqlStream::connect(options, d)?;

        // https://dev.mysql.com/doc/dev/mysql-server/8.0.12/page_protocol_connection_phase.html
        // https://mariadb.com/kb/en/connection/

        let handshake: Handshake = stream.recv_packet()?.decode()?;

        let mut plugin = handshake.auth_plugin;
        let mut nonce = handshake.auth_plugin_data;

        // FIXME: server version parse is a bit ugly
        // expecting MAJOR.MINOR.PATCH

        let mut server_version = handshake.server_version.split('.');

        let server_version_major: u16 = server_version
            .next()
            .unwrap_or_default()
            .parse()
            .unwrap_or(0);

        let server_version_minor: u16 = server_version
            .next()
            .unwrap_or_default()
            .parse()
            .unwrap_or(0);

        let server_version_patch: u16 = server_version
            .next()
            .unwrap_or_default()
            .parse()
            .unwrap_or(0);

        stream.server_version = (
            server_version_major,
            server_version_minor,
            server_version_patch,
        );

        stream.capabilities &= handshake.server_capabilities;
        stream.capabilities |= Capabilities::PROTOCOL_41;

        if matches!(options.ssl_mode, MySqlSslMode::Disabled) {
            // remove the SSL capability if SSL has been explicitly disabled
            stream.capabilities.remove(Capabilities::SSL);
        }

        // Upgrade to TLS if we were asked to and the server supports it
        tls::maybe_upgrade(&mut stream, options)?;

        let auth_response = if let (Some(plugin), Some(password)) = (plugin, &options.password) {
            Some(plugin.scramble(&mut stream, password, &nonce)?)
        } else {
            None
        };

        stream.write_packet(HandshakeResponse {
            collation: stream.collation as u8,
            max_packet_size: MAX_PACKET_SIZE,
            username: &options.username,
            database: options.database.as_deref(),
            auth_plugin: plugin,
            auth_response: auth_response.as_deref(),
        });

        stream.flush()?;

        loop {
            let packet = stream.recv_packet()?;
            match packet[0] {
                0x00 => {
                    let _ok = packet.ok()?;

                    break;
                }

                0xfe => {
                    let switch: AuthSwitchRequest = packet.decode()?;

                    plugin = Some(switch.plugin);
                    nonce = switch.data.chain(Bytes::new());

                    let response = switch
                        .plugin
                        .scramble(
                            &mut stream,
                            options.password.as_deref().unwrap_or_default(),
                            &nonce,
                        )
                        ?;

                    stream.write_packet(AuthSwitchResponse(response));
                    stream.flush()?;
                }

                id => {
                    if let (Some(plugin), Some(password)) = (plugin, &options.password) {
                        if plugin.handle(&mut stream, packet, password, &nonce)? {
                            // plugin signaled authentication is ok
                            break;
                        }

                        // plugin signaled to continue authentication
                    } else {
                        return Err(err_protocol!(
                            "unexpected packet 0x{:02x} during authentication",
                            id
                        ));
                    }
                }
            }
        }

        Ok(Self {
            stream,
            transaction_depth: 0,
            cache_statement: StatementCache::new(options.statement_cache_capacity),
        })
    }
}

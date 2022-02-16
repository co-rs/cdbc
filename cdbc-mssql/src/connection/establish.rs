use cdbc::Error;
use cdbc::utils::statement_cache::StatementCache;
use cdbc::io::Decode;
use crate::connection::stream::MssqlStream;
use crate::protocol::login::Login7;
use crate::protocol::message::Message;
use crate::protocol::packet::PacketType;
use crate::protocol::pre_login::{Encrypt, PreLogin, Version};
use crate::{MssqlConnectOptions, MssqlConnection};

impl MssqlConnection {
    pub fn establish(options: &MssqlConnectOptions) -> Result<Self, Error> {
        let mut stream: MssqlStream = MssqlStream::connect(options)?;

        // Send PRELOGIN to set up the context for login. The server should immediately
        // respond with a PRELOGIN message of its own.

        // TODO: Encryption
        // TODO: Send the version of SQLx over

        stream.write_packet(
            PacketType::PreLogin,
            PreLogin {
                version: Version::default(),
                encryption: Encrypt::NOT_SUPPORTED,

                ..Default::default()
            },
        );

        stream.flush()?;

        let (_, packet) = stream.recv_packet()?;
        let _ = PreLogin::decode(packet)?;

        // LOGIN7 defines the authentication rules for use between client and server

        stream.write_packet(
            PacketType::Tds7Login,
            Login7 {
                // FIXME: use a version constant
                version: 0x74000004, // SQL Server 2012 - SQL Server 2019
                client_program_version: 0,
                client_pid: 0,
                packet_size: 4096,
                hostname: "",
                username: &options.username,
                password: options.password.as_deref().unwrap_or_default(),
                app_name: "",
                server_name: "",
                client_interface_name: "",
                language: "",
                database: &*options.database,
                client_id: [0; 6],
            },
        );

        stream.flush()?;

        loop {
            // NOTE: we should receive an [Error] message if something goes wrong, otherwise,
            //       all messages are mostly informational (ENVCHANGE, INFO, LOGINACK)

            match stream.recv_message()? {
                Message::LoginAck(_) => {
                    // indicates that the login was successful
                    // no action is needed, we are just going to keep waiting till we hit <Done>
                }

                Message::Done(_) => {
                    break;
                }

                _ => {}
            }
        }

        // FIXME: Do we need to expose the capacity count here? It's not tied to
        //        server-side resources but just .prepare() calls which return
        //        client-side data.

        Ok(Self {
            stream,
            cache_statement: StatementCache::new(1024),
        })
    }
}

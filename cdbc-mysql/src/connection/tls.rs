use cdbc::error::Error;
use crate::connection::MySqlStream;
use crate::protocol::connect::SslRequest;
use crate::protocol::Capabilities;
use crate::{MySqlConnectOptions, MySqlSslMode};

pub(super) fn maybe_upgrade(
    stream: &mut MySqlStream,
    options: &MySqlConnectOptions,
) -> Result<(), Error> {
    // https://www.postgresql.org/docs/12/libpq-ssl.html#LIBPQ-SSL-SSLMODE-STATEMENTS
    match options.ssl_mode {
        MySqlSslMode::Disabled => {}

        MySqlSslMode::Preferred => {
            // try upgrade, but its okay if we fail
            upgrade(stream, options)?;
        }

        MySqlSslMode::Required | MySqlSslMode::VerifyIdentity | MySqlSslMode::VerifyCa => {
            if !upgrade(stream, options)? {
                // upgrade failed, die
                return Err(Error::Tls("server does not support TLS".into()));
            }
        }
    }

    Ok(())
}

fn upgrade(stream: &mut MySqlStream, options: &MySqlConnectOptions) -> Result<bool, Error> {
    if !stream.capabilities.contains(Capabilities::SSL) {
        // server does not support TLS
        return Ok(false);
    }

    stream.write_packet(SslRequest {
        max_packet_size: super::MAX_PACKET_SIZE,
        collation: stream.collation as u8,
    });

    stream.flush()?;

    let accept_invalid_certs = !matches!(
        options.ssl_mode,
        MySqlSslMode::VerifyCa | MySqlSslMode::VerifyIdentity
    );
    let accept_invalid_host_names = !matches!(options.ssl_mode, MySqlSslMode::VerifyIdentity);

    if !cfg!(feature = "native-tls")  {
        return Result::Err(Error::from("must enable native-tls!"));
    }
    #[cfg(feature = "native-tls")]
    stream
        .upgrade(
            &options.host,
            accept_invalid_certs,
            accept_invalid_host_names,
            options.ssl_ca.as_ref(),
        )
        ?;

    Ok(true)
}

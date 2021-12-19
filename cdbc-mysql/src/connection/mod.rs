use cdbc::utils::statement_cache::StatementCache;
use cdbc::connection::{Connection};
use cdbc::error::Error;
use crate::protocol::statement::StmtClose;
use crate::protocol::text::{Ping, Quit};
use crate::statement::MySqlStatementMetadata;
use crate::{MySql, MySqlConnectOptions};
use cdbc::transaction::Transaction;
use std::fmt::{self, Debug, Formatter};

mod auth;
mod establish;
mod executor;
mod stream;
mod tls;

pub(crate) use stream::{MySqlStream, Waiting};


const MAX_PACKET_SIZE: u32 = 1024;

/// A connection to a MySQL database.
pub struct MySqlConnection {
    // underlying TCP stream,
    // wrapped in a potentially TLS stream,
    // wrapped in a buffered stream
    pub(crate) stream: MySqlStream,

    // transaction status
    pub(crate) transaction_depth: usize,

    // cache by query string to the statement id and metadata
    cache_statement: StatementCache<(u32, MySqlStatementMetadata)>,
}

impl Debug for MySqlConnection {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("MySqlConnection").finish()
    }
}

impl Connection for MySqlConnection {

    type Options = MySqlConnectOptions;

    fn close(mut self) -> Result<(), Error> {
        {
            self.stream.send_packet(Quit)?;
            self.stream.shutdown()?;

            Ok(())
        }
    }

    fn ping(&mut self) -> Result<(), Error> {
        self.stream.wait_until_ready()?;
        self.stream.send_packet(Ping)?;
        self.stream.recv_ok()?;

        Ok(())
    }

    #[doc(hidden)]
    fn flush(&mut self) -> Result<(), Error> {
        self.stream.wait_until_ready()
    }

    fn cached_statements_size(&self) -> usize {
        self.cache_statement.len()
    }

    fn clear_cached_statements(&mut self) -> Result<(), Error> {
        while let Some((statement_id, _)) = self.cache_statement.remove_lru() {
            self.stream
                .send_packet(StmtClose {
                    statement: statement_id,
                })
                ?;
        }
        Ok(())
    }

    #[doc(hidden)]
    fn should_flush(&self) -> bool {
        !self.stream.wbuf.is_empty()
    }

    fn begin(&mut self) -> Result<Transaction<'_, Self::Database>, Error>
        where
            Self: Sized,
    {

        Transaction::begin(self)
    }
}

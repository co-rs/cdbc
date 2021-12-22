use std::fmt::{self, Debug, Formatter};
use std::sync::Arc;

use cdbc::HashMap;
use cdbc::utils::statement_cache::StatementCache;
use cdbc::connection::{Connection};
use cdbc::error::Error;
use cdbc::executor::Executor;
use cdbc::utils::ustr::UStr;
use cdbc::io::Decode;
use crate::message::{
    Close, Message, MessageFormat, ReadyForQuery, Terminate, TransactionStatus,
};
use crate::statement::PgStatementMetadata;
use crate::{PgConnectOptions, PgTypeInfo, Postgres};
use cdbc::transaction::Transaction;

pub use self::stream::PgStream;

pub(crate) mod describe;
mod establish;
mod executor;
mod sasl;
mod stream;
mod tls;

/// A connection to a PostgreSQL database.
pub struct PgConnection {
    // underlying TCP or UDS stream,
    // wrapped in a potentially TLS stream,
    // wrapped in a buffered stream
    pub(crate) stream: PgStream,

    // process id of this backend
    // used to send cancel requests
    #[allow(dead_code)]
    process_id: u32,

    // secret key of this backend
    // used to send cancel requests
    #[allow(dead_code)]
    secret_key: u32,

    // sequence of statement IDs for use in preparing statements
    // in PostgreSQL, the statement is prepared to a user-supplied identifier
    next_statement_id: u32,

    // cache statement by query string to the id and columns
    cache_statement: StatementCache<(u32, Arc<PgStatementMetadata>)>,

    // cache user-defined types by id <-> info
    cache_type_info: HashMap<u32, PgTypeInfo>,
    cache_type_oid: HashMap<UStr, u32>,

    // number of ReadyForQuery messages that we are currently expecting
    pub(crate) pending_ready_for_query_count: usize,

    // current transaction status
    transaction_status: TransactionStatus,
    pub(crate) transaction_depth: usize,
}

impl PgConnection {
    // will return when the connection is ready for another query
    pub fn wait_until_ready(&mut self) -> Result<(), Error> {
        if !self.stream.wbuf.is_empty() {
            self.stream.flush()?;
        }

        while self.pending_ready_for_query_count > 0 {
            let message = self.stream.recv()?;

            if let MessageFormat::ReadyForQuery = message.format {
                self.handle_ready_for_query(message)?;
            }
        }

        Ok(())
    }

    fn recv_ready_for_query(&mut self) -> Result<(), Error> {
        let r: ReadyForQuery = self
            .stream
            .recv_expect(MessageFormat::ReadyForQuery)
            ?;

        self.pending_ready_for_query_count -= 1;
        self.transaction_status = r.transaction_status;

        Ok(())
    }

    fn handle_ready_for_query(&mut self, message: Message) -> Result<(), Error> {
        self.pending_ready_for_query_count -= 1;
        self.transaction_status = ReadyForQuery::decode(message.contents)?.transaction_status;

        Ok(())
    }
}

impl Debug for PgConnection {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("PgConnection").finish()
    }
}

impl Connection for PgConnection {

    type Options = PgConnectOptions;

    fn close(mut self) -> Result<(), Error> {
        // The normal, graceful termination procedure is that the frontend sends a Terminate
        // message and immediately closes the connection.

        // On receipt of this message, the backend closes the
        // connection and terminates.

            self.stream.send(Terminate)?;
            self.stream.shutdown()?;

            Ok(())
    }

    fn ping(&mut self) ->  Result<(), Error> {
        // By sending a comment we avoid an error if the connection was in the middle of a rowset
        self.execute("/* SQLx ping */")?;
        Ok(())
    }

    fn begin(&mut self) ->  Result<Transaction<'_, Self::Database>, Error>
    where
        Self: Sized,
    {
        Transaction::begin(self)
    }

    fn cached_statements_size(&self) -> usize {
        self.cache_statement.len()
    }

    fn clear_cached_statements(&mut self) ->  Result<(), Error> {
            let mut cleared = 0_usize;

            self.wait_until_ready()?;

            while let Some((id, _)) = self.cache_statement.remove_lru() {
                self.stream.write(Close::Statement(id));
                cleared += 1;
            }

            if cleared > 0 {
                self.write_sync();
                self.stream.flush()?;

                self.wait_for_close_complete(cleared)?;
                self.recv_ready_for_query()?;
            }

            Ok(())
    }

    #[doc(hidden)]
    fn flush(&mut self) ->  Result<(), Error> {
        self.wait_until_ready()
    }

    #[doc(hidden)]
    fn should_flush(&self) -> bool {
        !self.stream.wbuf.is_empty()
    }
}

pub trait PgConnectionInfo {
    /// the version number of the server in `libpq` format
    fn server_version_num(&self) -> Option<u32>;
}

impl PgConnectionInfo for PgConnection {
    fn server_version_num(&self) -> Option<u32> {
        self.stream.server_version_num
    }
}

impl PgConnectionInfo for cdbc::pool::PoolConnection<Postgres> {
    fn server_version_num(&self) -> Option<u32> {
        self.stream.server_version_num
    }
}

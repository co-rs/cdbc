use cdbc::utils::statement_cache::StatementCache;
use cdbc::connection::{Connection};
use cdbc::executor::Executor;
use crate::connection::stream::MssqlStream;
use crate::statement::MssqlStatementMetadata;
use crate::{Mssql, MssqlConnectOptions};
use cdbc::transaction::Transaction;
use std::fmt::{self, Debug, Formatter};
use std::net::Shutdown;
use std::sync::Arc;
use either::Either;
use cdbc::database::{Database, HasStatement};
use cdbc::describe::Describe;
use cdbc::Execute;
use cdbc::io::chan_stream::ChanStream;

mod establish;
mod executor;
mod prepare;
mod stream;

pub struct MssqlConnection {
    pub stream: MssqlStream,
    pub cache_statement: StatementCache<Arc<MssqlStatementMetadata>>,
}

impl Debug for MssqlConnection {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("MssqlConnection").finish()
    }
}

impl Executor for MssqlConnection {
    type Database = Mssql;

    fn fetch_many<'q, E: 'q>(&mut self, query: E) -> ChanStream<Either<<Self::Database as Database>::QueryResult, <Self::Database as Database>::Row>> where E: Execute<'q, Self::Database> {
         self.fetch_many(query)
    }

    fn fetch_optional<'q, E: 'q>(&mut self, query: E) -> Result<Option<<Self::Database as Database>::Row>, cdbc::Error> where E: Execute<'q, Self::Database> {
        self.fetch_optional(query)
    }

    fn prepare_with<'q>(&mut self, sql: &'q str, parameters: &'q [<Self::Database as Database>::TypeInfo]) -> Result<<Self::Database as HasStatement<'q>>::Statement, cdbc::Error> {
        self.prepare_with(sql,parameters)
    }

    fn describe(&mut self, sql: &str) -> Result<Describe<Self::Database>, cdbc::Error> {
        self.describe(sql)
    }
}

impl Connection for MssqlConnection {

    type Options = MssqlConnectOptions;

    #[allow(unused_mut)]
    fn close(mut self) ->  Result<(), cdbc::Error> {
       Ok(self.stream.shutdown(Shutdown::Both)?)
    }

    fn ping(&mut self) -> Result<(), cdbc::Error> {
        // NOTE: we do not use `SELECT 1` as that *could* interact with any ongoing transactions
        self.execute("/* SQLx ping */")?;
        Ok(())
    }

    fn begin(&mut self) ->  Result<Transaction<'_, Self::Database>, cdbc::Error>
    where
        Self: Sized,
    {
        Ok(Transaction::begin(self)?)
    }

    #[doc(hidden)]
    fn flush(&mut self) ->  Result<(), cdbc::Error> {
        self.stream.wait_until_ready()
    }

    #[doc(hidden)]
    fn should_flush(&self) -> bool {
        !self.stream.wbuf.is_empty()
    }
}

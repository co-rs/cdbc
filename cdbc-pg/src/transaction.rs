
use cdbc::error::Error;
use cdbc::executor::Executor;
use crate::message::Query;
use crate::{PgConnection, Postgres};
use cdbc::transaction::{
    begin_ansi_transaction_sql, commit_ansi_transaction_sql, rollback_ansi_transaction_sql,
    TransactionManager,
};

/// Implementation of [`TransactionManager`] for PostgreSQL.
pub struct PgTransactionManager;

impl TransactionManager for PgTransactionManager {
    type Database = Postgres;

    fn begin(conn: &mut PgConnection) ->  Result<(), Error> {

            conn.execute(&*begin_ansi_transaction_sql(conn.transaction_depth))
                ?;

            conn.transaction_depth += 1;

            Ok(())

    }

    fn commit(conn: &mut PgConnection) ->  Result<(), Error> {
            if conn.transaction_depth > 0 {
                conn.execute(&*commit_ansi_transaction_sql(conn.transaction_depth))?;
                conn.transaction_depth -= 1;
            }
            Ok(())

    }

    fn rollback(conn: &mut PgConnection) ->  Result<(), Error> {
            if conn.transaction_depth > 0 {
                conn.execute(&*rollback_ansi_transaction_sql(conn.transaction_depth))
                    ?;
                conn.transaction_depth -= 1;
            }
            Ok(())
    }

    fn start_rollback(conn: &mut PgConnection) {
        if conn.transaction_depth > 0 {
            conn.pending_ready_for_query_count += 1;
            conn.stream.write(Query(&rollback_ansi_transaction_sql(
                conn.transaction_depth,
            )));

            conn.transaction_depth -= 1;
        }
    }
}

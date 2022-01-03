use std::ptr;

use libsqlite3_sys::{sqlite3_exec, SQLITE_OK};

use cdbc::error::Error;
use cdbc::executor::Executor;
use crate::{Sqlite, SqliteConnection, SqliteError};
use cdbc::transaction::{
    begin_ansi_transaction_sql, commit_ansi_transaction_sql, rollback_ansi_transaction_sql,
    TransactionManager,
};

/// Implementation of [`TransactionManager`] for SQLite.
pub struct SqliteTransactionManager;

impl TransactionManager for SqliteTransactionManager {
    type Database = Sqlite;

    fn begin(conn: &mut SqliteConnection) ->  Result<(), Error> {
      conn.worker.begin()
    }

    fn commit(conn: &mut SqliteConnection) ->Result<(), Error> {
       conn.worker.commit()
    }

    fn rollback(conn: &mut SqliteConnection) ->  Result<(), Error> {
       conn.worker.rollback()
    }

    fn start_rollback(conn: &mut SqliteConnection) {
        conn.worker.start_rollback().ok();
    }
}

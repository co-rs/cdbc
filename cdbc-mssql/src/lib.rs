//! Microsoft SQL (MSSQL) database driver.
//!
#[macro_use]
extern crate mco;
#[macro_use]
pub extern crate cdbc;

use cdbc::executor::Executor;

mod arguments;
mod column;
mod connection;
mod database;
mod error;
mod io;
mod options;
mod protocol;
mod query_result;
mod row;
mod statement;
mod transaction;
mod type_info;
pub mod types;
mod value;

pub use arguments::MssqlArguments;
pub use column::MssqlColumn;
pub use connection::MssqlConnection;
pub use database::Mssql;
pub use error::MssqlDatabaseError;
pub use options::MssqlConnectOptions;
pub use query_result::MssqlQueryResult;
pub use row::MssqlRow;
pub use statement::MssqlStatement;
pub use transaction::MssqlTransactionManager;
pub use type_info::MssqlTypeInfo;
pub use value::{MssqlValue, MssqlValueRef};

/// An alias for [`Pool`][crate::pool::Pool], specialized for MSSQL.
pub type MssqlPool = cdbc::pool::Pool<Mssql>;

/// An alias for [`PoolOptions`][crate::pool::PoolOptions], specialized for SQLite.
pub type MssqlPoolOptions = cdbc::pool::PoolOptions<Mssql>;

/// An alias for [`Executor<'_, Database = Mssql>`][Executor].
pub trait MssqlExecutor<'c>: Executor<Database = Mssql> {}
impl<'c, T: Executor<Database = Mssql>> MssqlExecutor<'c> for T {}

// NOTE: required due to the lack of lazy normalization
impl_into_arguments_for_arguments!(MssqlArguments);
impl_column_index_for_row!(MssqlRow);
impl_column_index_for_statement!(MssqlStatement);
impl_into_maybe_pool!(Mssql, MssqlConnection);

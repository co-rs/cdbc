use cdbc::database::{Database, HasArguments, HasStatement, HasStatementCache, HasValueRef};
use crate::{
    SqliteArgumentValue, SqliteArguments, SqliteColumn, SqliteConnection, SqliteQueryResult,
    SqliteRow, SqliteStatement, SqliteTransactionManager, SqliteTypeInfo, SqliteValue,
    SqliteValueRef,
};

/// Sqlite database driver.
#[derive(Debug)]
pub struct Sqlite;

impl Database for Sqlite {
    type Connection = SqliteConnection;

    type TransactionManager = SqliteTransactionManager;

    type Row = SqliteRow;

    type QueryResult = SqliteQueryResult;

    type Column = SqliteColumn;

    type TypeInfo = SqliteTypeInfo;

    type Value = SqliteValue;

    fn holder() -> &'static str {
        "?"
    }
}

impl<'r> HasValueRef<'r> for Sqlite {
    type Database = Sqlite;

    type ValueRef = SqliteValueRef<'r>;
}

impl<'q> HasArguments<'q> for Sqlite {
    type Database = Sqlite;

    type Arguments = SqliteArguments<'q>;

    type ArgumentBuffer = Vec<SqliteArgumentValue<'q>>;
}

impl HasStatement for Sqlite {
    type Database = Sqlite;

    type Statement = SqliteStatement;
}

impl HasStatementCache for Sqlite {}

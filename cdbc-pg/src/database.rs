use cdbc::database::{Database, HasArguments, HasStatement, HasStatementCache, HasValueRef};
use crate::arguments::PgArgumentBuffer;
use crate::value::{PgValue, PgValueRef};
use crate::{
    PgArguments, PgColumn, PgConnection, PgQueryResult, PgRow, PgStatement, PgTransactionManager,
    PgTypeInfo,
};

/// PostgreSQL database driver.
#[derive(Debug)]
pub struct Postgres;

impl Database for Postgres {
    type Connection = PgConnection;

    type TransactionManager = PgTransactionManager;

    type Row = PgRow;

    type QueryResult = PgQueryResult;

    type Column = PgColumn;

    type TypeInfo = PgTypeInfo;

    type Value = PgValue;

    fn holder() -> &'static str {
        "$"
    }
}

impl<'r> HasValueRef<'r> for Postgres {
    type Database = Postgres;

    type ValueRef = PgValueRef<'r>;
}

impl HasArguments<'_> for Postgres {
    type Database = Postgres;

    type Arguments = PgArguments;

    type ArgumentBuffer = PgArgumentBuffer;
}

impl HasStatement for Postgres {
    type Database = Postgres;

    type Statement = PgStatement;
}

impl HasStatementCache for Postgres {}

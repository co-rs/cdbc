use cdbc::database::{Database, HasArguments, HasStatement, HasValueRef};
use crate::{
    MssqlArguments, MssqlColumn, MssqlConnection, MssqlQueryResult, MssqlRow, MssqlStatement,
    MssqlTransactionManager, MssqlTypeInfo, MssqlValue, MssqlValueRef,
};

/// MSSQL database driver.
#[derive(Debug)]
pub struct Mssql;

impl Database for Mssql {
    type Connection = MssqlConnection;

    type TransactionManager = MssqlTransactionManager;

    type Row = MssqlRow;

    type QueryResult = MssqlQueryResult;

    type Column = MssqlColumn;

    type TypeInfo = MssqlTypeInfo;

    type Value = MssqlValue;
}

impl<'r> HasValueRef<'r> for Mssql {
    type Database = Mssql;

    type ValueRef = MssqlValueRef<'r>;
}

impl HasStatement for Mssql {
    type Database = Mssql;

    type Statement = MssqlStatement;
}

impl HasArguments<'_> for Mssql {
    type Database = Mssql;

    type Arguments = MssqlArguments;

    type ArgumentBuffer = Vec<u8>;
}

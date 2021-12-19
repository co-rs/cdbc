use cdbc::connection::ConnectOptions;
use cdbc::error::Error;
use cdbc::executor::Executor;
use crate::connection::establish::establish;
use crate::{SqliteConnectOptions, SqliteConnection};
use std::time::Duration;

impl ConnectOptions for SqliteConnectOptions {
    type Connection = SqliteConnection;

    fn connect(&self) ->  Result<Self::Connection, Error>
    where
        Self::Connection: Sized,
    {
            let mut conn = establish(self)?;

            // send an initial sql statement comprised of options
            let mut init = String::new();

            for (key, value) in self.pragmas.iter() {
                use std::fmt::Write;
                write!(init, "PRAGMA {} = {}; ", key, value).ok();
            }

            conn.execute(&*init)?;

            Ok(conn)
    }
}

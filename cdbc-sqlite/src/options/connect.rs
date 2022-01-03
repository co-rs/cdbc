use cdbc::connection::ConnectOptions;
use cdbc::error::Error;
use cdbc::executor::Executor;
use crate::connection::establish::establish;
use crate::{SqliteConnectOptions, SqliteConnection};
use std::time::Duration;
use std::fmt::Write;

impl ConnectOptions for SqliteConnectOptions {
    type Connection = SqliteConnection;

    fn connect(&self) -> Result<Self::Connection, Error>
        where
            Self::Connection: Sized,
    {
        let mut conn = establish(self)?;

        // send an initial sql statement comprised of options
        let mut init = String::new();

        // This is a special case for sqlcipher. When the `key` pragma
        // is set, we have to make sure it's executed first in order.
        if let Some(pragma_key_password) = self.pragmas.get("key") {
            write!(init, "PRAGMA key = {}; ", pragma_key_password).ok();
        }

        for (key, value) in self.pragmas.iter() {
            // Since we've already written the possible `key` pragma
            // above, we shall skip it now.
            if key == "key" {
                continue;
            }
            write!(init, "PRAGMA {} = {}; ", key, value).ok();
        }

        conn.execute(&*init)?;

        Ok(conn)
    }
}

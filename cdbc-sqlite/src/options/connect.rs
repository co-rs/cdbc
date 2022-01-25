use cdbc::connection::ConnectOptions;
use cdbc::error::Error;
use cdbc::executor::Executor;
use crate::{SqliteConnectOptions, SqliteConnection};
use std::time::Duration;
use std::fmt::Write;

impl ConnectOptions for SqliteConnectOptions {
    type Connection = SqliteConnection;

    fn connect(&self,d:Duration) -> Result<Self::Connection, Error>
        where
            Self::Connection: Sized,
    {
            let mut conn = SqliteConnection::establish(self)?;

            // send an initial sql statement comprised of options
            let mut init = String::new();

            // This is a special case for sqlcipher. When the `key` pragma
            // is set, we have to make sure it's executed first in order.
            if let Some(pragma_key_password) = self.pragmas.get("key") {
                write!(init, "PRAGMA key = {}; ", pragma_key_password).ok();
            }

            for (key, value) in &self.pragmas {
                // Since we've already written the possible `key` pragma
                // above, we shall skip it now.
                if key == "key" {
                    continue;
                }
                write!(init, "PRAGMA {} = {}; ", key, value).ok();
            }

            conn.execute(&*init)?;

            if !self.collations.is_empty() {
                let mut locked = conn.lock_handle()?;

                for collation in &self.collations {
                    collation.create(&mut locked.guard.handle)?;
                }
            }

            Ok(conn)
    }
}

use std::time::Duration;
use cdbc::connection::ConnectOptions;
use cdbc::error::Error;
use crate::{PgConnectOptions, PgConnection};
impl ConnectOptions for PgConnectOptions {
    type Connection = PgConnection;

    fn connect(&self,d:Duration) -> Result<Self::Connection, Error>
    where
        Self::Connection: Sized,
    {
       PgConnection::establish(self,d)
    }
}

use cdbc::connection::ConnectOptions;
use cdbc::error::Error;
use crate::{PgConnectOptions, PgConnection};
impl ConnectOptions for PgConnectOptions {
    type Connection = PgConnection;

    fn connect(&self) -> Result<Self::Connection, Error>
    where
        Self::Connection: Sized,
    {
       PgConnection::establish(self)
    }
}

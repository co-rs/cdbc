use cdbc::connection::ConnectOptions;
use crate::{MssqlConnectOptions, MssqlConnection};
use std::time::Duration;
use cdbc::Error;

impl ConnectOptions for MssqlConnectOptions {
    type Connection = MssqlConnection;

    fn connect(&self,d:Duration) ->Result<Self::Connection, Error>
    where
        Self::Connection: Sized,
    {
        Ok(MssqlConnection::establish(self)?)
    }
}

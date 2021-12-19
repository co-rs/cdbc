use cdbc::connection::ConnectOptions;
use cdbc::error::Error;
use crate::{PgConnectOptions, PgConnection};
use log::LevelFilter;
use std::time::Duration;

impl ConnectOptions for PgConnectOptions {
    type Connection = PgConnection;

    fn connect(&self) -> Result<Self::Connection, Error>
    where
        Self::Connection: Sized,
    {
       PgConnection::establish(self)
    }

    fn log_statements(&mut self, level: LevelFilter) -> &mut Self {
        self.log_settings.log_statements(level);
        self
    }

    fn log_slow_statements(&mut self, level: LevelFilter, duration: Duration) -> &mut Self {
        self.log_settings.log_slow_statements(level, duration);
        self
    }
}

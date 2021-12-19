use cdbc::error::{Error, Result};
use cdbc::pool::{Pool, PoolConnection};
use crate::connection::PgConnection;
use crate::message::{
    CommandComplete, CopyData, CopyDone, CopyFail, CopyResponse, MessageFormat, Query,
};
use crate::Postgres;
use bytes::{BufMut, Bytes};
use smallvec::alloc::borrow::Cow;
use std::convert::TryFrom;
use std::ops::{Deref, DerefMut};
use cdbc::io::chan_stream::ChanStream;
use std::io::Write;

impl PgConnection {
    /// Issue a `COPY FROM STDIN` statement and transition the connection to streaming data
    /// to Postgres. This is a more efficient way to import data into Postgres as compared to
    /// `INSERT` but requires one of a few specific data formats (text/CSV/binary).
    ///
    /// If `statement` is anything other than a `COPY ... FROM STDIN ...` command, an error is
    /// returned.
    ///
    /// Command examples and accepted formats for `COPY` data are shown here:
    /// https://www.postgresql.org/docs/current/sql-copy.html
    ///
    /// ### Note
    /// [PgCopyIn::finish] or [PgCopyIn::abort] *must* be called when finished or the connection
    /// will return an error the next time it is used.
    pub fn copy_in_raw(&mut self, statement: &str) -> Result<PgCopyIn<&mut Self>> {
        PgCopyIn::begin(self, statement)
    }

    /// Issue a `COPY TO STDOUT` statement and transition the connection to streaming data
    /// from Postgres. This is a more efficient way to export data from Postgres but
    /// arrives in chunks of one of a few data formats (text/CSV/binary).
    ///
    /// If `statement` is anything other than a `COPY ... TO STDOUT ...` command,
    /// an error is returned.
    ///
    /// Note that once this process has begun, unless you read the stream to completion,
    /// it can only be canceled in two ways:
    ///
    /// 1. by closing the connection, or:
    /// 2. by using another connection to kill the server process that is sending the data as shown
    /// [in this StackOverflow answer](https://stackoverflow.com/a/35319598).
    ///
    /// If you don't read the stream to completion, the next time the connection is used it will
    /// need to read and discard all the remaining queued data, which could take some time.
    ///
    /// Command examples and accepted formats for `COPY` data are shown here:
    /// https://www.postgresql.org/docs/current/sql-copy.html
    #[allow(clippy::needless_lifetimes)]
    pub fn copy_out_raw<'c>(
        &'c mut self,
        statement: &str,
    ) -> Result<ChanStream<Bytes>> {
        pg_begin_copy_out(self, statement)
    }
}

pub trait CopyRaw{
     fn copy_in_raw(&self, statement: &str) -> Result<PgCopyIn<PoolConnection<Postgres>>>;
    fn copy_out_raw(&self, statement: &str) -> Result<ChanStream<Bytes>>;
}

impl CopyRaw for Pool<Postgres> {
    /// Issue a `COPY FROM STDIN` statement and begin streaming data to Postgres.
    /// This is a more efficient way to import data into Postgres as compared to
    /// `INSERT` but requires one of a few specific data formats (text/CSV/binary).
    ///
    /// A single connection will be checked out for the duration.
    ///
    /// If `statement` is anything other than a `COPY ... FROM STDIN ...` command, an error is
    /// returned.
    ///
    /// Command examples and accepted formats for `COPY` data are shown here:
    /// https://www.postgresql.org/docs/current/sql-copy.html
    ///
    /// ### Note
    /// [PgCopyIn::finish] or [PgCopyIn::abort] *must* be called when finished or the connection
    /// will return an error the next time it is used.
    fn copy_in_raw(&self, statement: &str) -> Result<PgCopyIn<PoolConnection<Postgres>>> {
        PgCopyIn::begin(self.acquire()?, statement)
    }

    /// Issue a `COPY TO STDOUT` statement and begin streaming data
    /// from Postgres. This is a more efficient way to export data from Postgres but
    /// arrives in chunks of one of a few data formats (text/CSV/binary).
    ///
    /// If `statement` is anything other than a `COPY ... TO STDOUT ...` command,
    /// an error is returned.
    ///
    /// Note that once this process has begun, unless you read the stream to completion,
    /// it can only be canceled in two ways:
    ///
    /// 1. by closing the connection, or:
    /// 2. by using another connection to kill the server process that is sending the data as shown
    /// [in this StackOverflow answer](https://stackoverflow.com/a/35319598).
    ///
    /// If you don't read the stream to completion, the next time the connection is used it will
    /// need to read and discard all the remaining queued data, which could take some time.
    ///
    /// Command examples and accepted formats for `COPY` data are shown here:
    /// https://www.postgresql.org/docs/current/sql-copy.html
    fn copy_out_raw(&self, statement: &str) -> Result<ChanStream<Bytes>> {
        pg_begin_copy_out(self.acquire()?, statement)
    }
}

/// A connection in streaming `COPY FROM STDIN` mode.
///
/// Created by [PgConnection::copy_in_raw] or [Pool::copy_out_raw].
///
/// ### Note
/// [PgCopyIn::finish] or [PgCopyIn::abort] *must* be called when finished or the connection
/// will return an error the next time it is used.
#[must_use = "connection will error on next use if `.finish()` or `.abort()` is not called"]
pub struct PgCopyIn<C: DerefMut<Target = PgConnection>> {
    conn: Option<C>,
    response: CopyResponse,
}

impl<C: DerefMut<Target = PgConnection>> PgCopyIn<C> {
    fn begin(mut conn: C, statement: &str) -> Result<Self> {
        conn.wait_until_ready()?;
        conn.stream.send(Query(statement))?;

        let response: CopyResponse = conn
            .stream
            .recv_expect(MessageFormat::CopyInResponse)
            ?;

        Ok(PgCopyIn {
            conn: Some(conn),
            response,
        })
    }

    /// Returns `true` if Postgres is expecting data in text or CSV format.
    pub fn is_textual(&self) -> bool {
        self.response.format == 0
    }

    /// Returns the number of columns expected in the input.
    pub fn num_columns(&self) -> usize {
        assert_eq!(
            self.response.num_columns as usize,
            self.response.format_codes.len(),
            "num_columns does not match format_codes.len()"
        );
        self.response.format_codes.len()
    }

    /// Check if a column is expecting data in text format (`true`) or binary format (`false`).
    ///
    /// ### Panics
    /// If `column` is out of range according to [`.num_columns()`][Self::num_columns].
    pub fn column_is_textual(&self, column: usize) -> bool {
        self.response.format_codes[column] == 0
    }

    /// Send a chunk of `COPY` data.
    ///
    /// If you're copying data from an `AsyncRead`, maybe consider [Self::read_from] instead.
    pub fn send(&mut self, data: impl Deref<Target = [u8]>) -> Result<&mut Self> {
        self.conn
            .as_deref_mut()
            .expect("send_data: conn taken")
            .stream
            .send(CopyData(data))
            ?;

        Ok(self)
    }

    /// Copy data directly from `source` to the database without requiring an intermediate buffer.
    ///
    /// `source` will be read to the end.
    ///
    /// ### Note
    /// You must still call either [Self::finish] or [Self::abort] to complete the process.
    pub fn read_from(&mut self, mut source: impl std::io::Read + Unpin) -> Result<&mut Self> {
        // this is a separate guard from WriteAndFlush so we can reuse the buffer without zeroing
        struct BufGuard<'s>(&'s mut Vec<u8>);

        impl Drop for BufGuard<'_> {
            fn drop(&mut self) {
                self.0.clear()
            }
        }

        let conn: &mut PgConnection = self.conn.as_deref_mut().expect("copy_from: conn taken");

        // flush any existing messages in the buffer and clear it
        conn.stream.flush()?;

        {
            let buf_stream = &mut *conn.stream;
            let stream = &mut buf_stream.stream;

            // ensures the buffer isn't left in an inconsistent state
            let mut guard = BufGuard(&mut buf_stream.wbuf);

            let buf: &mut Vec<u8> = &mut guard.0;
            buf.push(b'd'); // CopyData format code
            buf.resize(5, 0); // reserve space for the length

            loop {
                let read = match () {
                    _ => {
                        // should be a no-op unless len != capacity
                        buf.resize(buf.capacity(), 0);
                        source.read(&mut buf[5..])?
                    }
                };

                if read == 0 {
                    break;
                }

                let read32 = u32::try_from(read)
                    .map_err(|_| err_protocol!("number of bytes read exceeds 2^32: {}", read))?;

                (&mut buf[1..]).put_u32(read32 + 4);

                stream.write_all(&buf[..read + 5])?;
                stream.flush()?;
            }
        }

        Ok(self)
    }

    /// Signal that the `COPY` process should be aborted and any data received should be discarded.
    ///
    /// The given message can be used for indicating the reason for the abort in the database logs.
    ///
    /// The server is expected to respond with an error, so only _unexpected_ errors are returned.
    pub fn abort(mut self, msg: impl Into<String>) -> Result<()> {
        let mut conn = self
            .conn
            .take()
            .expect("PgCopyIn::fail_with: conn taken illegally");

        conn.stream.send(CopyFail::new(msg))?;

        match conn.stream.recv() {
            Ok(msg) => Err(err_protocol!(
                "fail_with: expected ErrorResponse, got: {:?}",
                msg.format
            )),
            Err(Error::Database(e)) => {
                match e.code() {
                    Some(Cow::Borrowed("57014")) => {
                        // postgres abort received error code
                        conn.stream
                            .recv_expect(MessageFormat::ReadyForQuery)
                            ?;
                        Ok(())
                    }
                    _ => Err(Error::Database(e)),
                }
            }
            Err(e) => Err(e),
        }
    }

    /// Signal that the `COPY` process is complete.
    ///
    /// The number of rows affected is returned.
    pub fn finish(mut self) -> Result<u64> {
        let mut conn = self
            .conn
            .take()
            .expect("CopyWriter::finish: conn taken illegally");

        conn.stream.send(CopyDone)?;
        let cc: CommandComplete = conn
            .stream
            .recv_expect(MessageFormat::CommandComplete)
            ?;

        conn.stream
            .recv_expect(MessageFormat::ReadyForQuery)
            ?;

        Ok(cc.rows_affected())
    }
}

impl<C: DerefMut<Target = PgConnection>> Drop for PgCopyIn<C> {
    fn drop(&mut self) {
        if let Some(mut conn) = self.conn.take() {
            conn.stream.write(CopyFail::new(
                "PgCopyIn dropped without calling finish() or fail()",
            ));
        }
    }
}

fn pg_begin_copy_out<'c, C: DerefMut<Target = PgConnection> + Send + 'c>(
    mut conn: C,
    statement: &str,
) -> Result<ChanStream<Bytes>> {
    conn.wait_until_ready()?;
    conn.stream.send(Query(statement))?;

    let _: CopyResponse = conn
        .stream
        .recv_expect(MessageFormat::CopyOutResponse)
        ?;

    let stream = chan_stream! {
        loop {
            let msg = conn.stream.recv()?;
            match msg.format {
                MessageFormat::CopyData => r#yield!(msg.decode::<CopyData<Bytes>>()?.0),
                MessageFormat::CopyDone => {
                    let _ = msg.decode::<CopyDone>()?;
                    conn.stream.recv_expect(MessageFormat::CommandComplete)?;
                    conn.stream.recv_expect(MessageFormat::ReadyForQuery)?;
                    return Ok(())
                },
                _ => return Err(err_protocol!("unexpected message format during copy out: {:?}", msg.format))
            }
        }
    };

    Ok(stream)
}

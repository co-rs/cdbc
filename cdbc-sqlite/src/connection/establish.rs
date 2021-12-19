use cdbc::error::Error;
use crate::connection::handle::ConnectionHandle;
use crate::statement::StatementWorker;
use crate::{SqliteConnectOptions, SqliteConnection, SqliteError};
use libsqlite3_sys::{
    sqlite3_busy_timeout, sqlite3_extended_result_codes, sqlite3_open_v2, SQLITE_OK,
    SQLITE_OPEN_CREATE, SQLITE_OPEN_FULLMUTEX, SQLITE_OPEN_MEMORY, SQLITE_OPEN_NOMUTEX,
    SQLITE_OPEN_PRIVATECACHE, SQLITE_OPEN_READONLY, SQLITE_OPEN_READWRITE, SQLITE_OPEN_SHAREDCACHE,
};
use std::io;
use std::{
    convert::TryFrom,
    ptr::{null, null_mut},
};
use cdbc::utils::statement_cache::StatementCache;

pub(crate) fn establish(options: &SqliteConnectOptions) -> Result<SqliteConnection, Error> {
    let mut filename = options
        .filename
        .to_str()
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "filename passed to SQLite must be valid UTF-8",
            )
        })?
        .to_owned();

    // By default, we connect to an in-memory database.
    // [SQLITE_OPEN_NOMUTEX] will instruct [sqlite3_open_v2] to return an error if it
    // cannot satisfy our wish for a thread-safe, lock-free connection object

    let mut flags = if options.serialized {
        SQLITE_OPEN_FULLMUTEX
    } else {
        SQLITE_OPEN_NOMUTEX
    };

    flags |= if options.read_only {
        SQLITE_OPEN_READONLY
    } else if options.create_if_missing {
        SQLITE_OPEN_CREATE | SQLITE_OPEN_READWRITE
    } else {
        SQLITE_OPEN_READWRITE
    };

    if options.in_memory {
        flags |= SQLITE_OPEN_MEMORY;
    }

    flags |= if options.shared_cache {
        SQLITE_OPEN_SHAREDCACHE
    } else {
        SQLITE_OPEN_PRIVATECACHE
    };

    if options.immutable {
        filename = format!("file:{}?immutable=true", filename);
        flags |= libsqlite3_sys::SQLITE_OPEN_URI;
    }

    filename.push('\0');

    let busy_timeout = options.busy_timeout;

    let handle = {
        let mut handle = null_mut();

        // <https://www.sqlite.org/c3ref/open.html>
        let mut status = unsafe {
            sqlite3_open_v2(
                filename.as_bytes().as_ptr() as *const _,
                &mut handle,
                flags,
                null(),
            )
        };

        if handle.is_null() {
            // Failed to allocate memory
            panic!("SQLite is unable to allocate memory to hold the sqlite3 object");
        }

        // SAFE: tested for NULL just above
        // This allows any returns below to close this handle with RAII
        let handle = unsafe { ConnectionHandle::new(handle) };

        if status != SQLITE_OK {
            return Err(Error::Database(Box::new(SqliteError::new(handle.as_ptr()))));
        }

        // Enable extended result codes
        // https://www.sqlite.org/c3ref/extended_result_codes.html
        unsafe {
            // NOTE: ignore the failure here
            sqlite3_extended_result_codes(handle.as_ptr(), 1);
        }

        // Configure a busy timeout
        // This causes SQLite to automatically sleep in increasing intervals until the time
        // when there is something locked during [sqlite3_step]. This is sync. but we only
        // run [sqlite3_step] in [blocking!] so its okay.
        //
        // We also need to convert the u128 value to i32, checking we're not overflowing.
        let ms =
            i32::try_from(busy_timeout.as_millis()).expect("Given busy timeout value is too big.");

        status = unsafe { sqlite3_busy_timeout(handle.as_ptr(), ms) };

        if status != SQLITE_OK {
            return Err(Error::Database(Box::new(SqliteError::new(handle.as_ptr()))));
        }
        Result::<ConnectionHandle, Error>::Ok(handle)
    };

   let handle = handle?;

    Ok(SqliteConnection {
        worker: StatementWorker::new(handle.to_ref()),
        handle,
        statements: StatementCache::new(options.statement_cache_capacity),
        statement: None,
        transaction_depth: 0,
    })
}

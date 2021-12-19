# cdbc
Database driver based on coroutine of [may](https://github.com/Xudong-Huang/may)

* High concurrency，based on coroutine
* No Future ，No ``` async fn ```, No ```.await ```, no Poll* func，No ```Pin``` 
* NativeTls and TCP connections are supported
* Low coupling，The database driver and the abstraction layer are designed separately
* Inspired by golang, [may](https://github.com/Xudong-Huang/may), sqlx



## Note: CDBC is the driver abstraction. For details, use the cDB-mysql or CDB-Postgres sublibraries


use example:

> cargo.toml
```toml
cdbc = {path = "../"}
cdbc-mysql = {path = "../cdbc-mysql"}
```
> main.rs
```rust
use std::collections::{BTreeMap, HashMap};
use cdbc::database::Database;
use cdbc_mysql::{MySql, MySqlPool, MySqlRow};
use cdbc::column::Column;
use cdbc::decode::Decode;
use cdbc::executor::Executor;
use cdbc::io::chan_stream::{ChanStream, Stream, TryStream};
use cdbc::query::Query;
use cdbc::row::Row;

fn main() -> cdbc::Result<()> {
    let pool = MySqlPool::connect("mysql://root:123456@localhost:3306/test")?;
    let mut conn = pool.acquire()?;
    loop {
        let mut data: ChanStream<_> = conn.fetch("select * from biz_activity;");
        data.try_for_each(|item| {
            let mut m = BTreeMap::new();
            let it: MySqlRow = item;
            for column in it.columns() {
                let v = it.try_get_raw(column.name())?;
                let r: Option<String> = Decode::<'_, MySql>::decode(v)?;
                m.insert(column.name().to_string(), r);
            }
            println!("{:?}", m);
            drop(m);
            Ok(())
        })?;
    }
}

```

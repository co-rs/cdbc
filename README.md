# cdbc
Coroutine([may](https://github.com/Xudong-Huang/may) Database driver Connectivity)

* High concurrency，based on coroutine
* No ``` Future<'q,Output=*> ```，No ``` async fn ```, No ```.await ```, no Poll* func，No ```Pin``` 
* Optimize the trait system so that it has intelligent hints of the base method
* NativeTls and TCP connections are supported
* Low coupling，The database driver and the abstraction layer are designed separately
* Inspired by golang, [may](https://github.com/Xudong-Huang/may), sqlx

# support database-driver
* cdbc-mysql  (done)
* cdbc-pg     (done)
* cdbc-sqlite (done)


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
        let mut data: ChanStream<MySqlRow> = conn.fetch("select * from biz_activity;");
        data.try_for_each(|item| {
            let mut m = BTreeMap::new();
            for column in item.columns() {
                let v = item.try_get_raw(column.name())?;
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

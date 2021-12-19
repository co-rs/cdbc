use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use log::Level;
use cdbc::database::Database;
use cdbc_sqlite::{Sqlite, SqlitePool, SqliteRow};
use cdbc::column::Column;
use cdbc::decode::Decode;
use cdbc::executor::Executor;
use cdbc::io::chan_stream::{ChanStream, Stream, TryStream};
use cdbc::query::Query;
use cdbc::row::Row;


fn main() -> cdbc::Result<()> {
    //first. create sqlite dir/file
    std::fs::create_dir_all("target/db/");
    let file=File::create("target/db/sqlite.db");
    drop(file);
    //next create table and query result
    let pool = SqlitePool::connect("sqlite://target/db/sqlite.db")?;
    let mut conn = pool.acquire()?;
    conn.execute("CREATE TABLE biz_activity(
   id string,
   name string,
   age int,
   delete_flag int)
   ");
    conn.execute("INSERT INTO biz_activity (id,name,age,delete_flag) values (\"1\",\"1\",1,0)");

    loop {
        let mut data: ChanStream<SqliteRow> = conn.fetch("select * from biz_activity;");
        data.try_for_each(|item| {
            let mut m = BTreeMap::new();
            for column in item.columns() {
                let v = item.try_get_raw(column.name())?;
                let r: Option<String> = Decode::<'_, Sqlite>::decode(v)?;
                m.insert(column.name().to_string(), r);
            }
            println!("{:?}", m);
            drop(m);
            Ok(())
        })?;
    }
}
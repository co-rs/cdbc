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
    let pool = make_sqlite().unwrap();
    //next create table and query result
    let mut conn = pool.acquire()?;
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

fn make_sqlite() -> cdbc::Result<SqlitePool> {
    //first. create sqlite dir/file
    std::fs::create_dir_all("target/db/");
    File::create("target/db/sqlite.db");
    //next create table and query result
    let pool = SqlitePool::connect("sqlite://target/db/sqlite.db")?;
    let mut conn = pool.acquire()?;
    conn.execute("CREATE TABLE biz_activity(  id string, name string,age int, delete_flag int) ");
    conn.execute("INSERT INTO biz_activity (id,name,age,delete_flag) values (\"1\",\"1\",1,0)");
    Ok(pool)
}

#[cfg(test)]
mod test {
    use cdbc::executor::Executor;
    use cdbc_sqlite::SqlitePool;
    use crate::make_sqlite;


    #[test]
    fn test_prepare_sql() -> cdbc::Result<()> {
        let pool = make_sqlite()?;
        #[derive(Debug)]
        pub struct BizActivity {
            pub id: Option<String>,
            pub name: Option<String>,
            pub delete_flag: Option<i32>,
        }
        let mut conn = pool.acquire()?;
        let mut q = cdbc::query::query("select * from biz_activity where id = ?");
        q = q.bind("1");
        let r = conn.fetch_all(q)?;
        let data = cdbc::row_scan_structs!(r,BizActivity{id:None,name:None,delete_flag:None})?;
        println!("{:?}", data);
        Ok(())
    }
}
use std::fs::File;
use cdbc::Executor;
use cdbc_sqlite::SqlitePool;

fn main() -> cdbc::Result<()> {
    let pool = make_sqlite()?;
    #[derive(Debug)]
    pub struct BizActivity {
        pub id: Option<String>,
        pub name: Option<String>,
        pub delete_flag: Option<i32>,
    }
    let data = cdbc::row_scans!(cdbc::query("select * from biz_activity where id = ?")
        .bind("1").fetch_all(pool.clone())?,BizActivity{id:None,name:None,delete_flag:None})?;
    println!("{:?}", data);
    Ok(())
}

fn make_sqlite() -> cdbc::Result<SqlitePool> {
    //first. create sqlite dir/file
    std::fs::create_dir_all("target/db/");
    File::create("../../../target/db/sqlite.db");
    //next create table and query result
    let pool = SqlitePool::connect("sqlite://target/db/sqlite.db")?;
    let mut conn = pool.acquire()?;
    conn.execute("CREATE TABLE biz_activity(  id string, name string,age int, delete_flag int) ");
    conn.execute("INSERT INTO biz_activity (id,name,age,delete_flag) values (\"1\",\"1\",1,0)");
    Ok(pool)
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;
    use cdbc::{Column, Decode, Executor, Row};
    use cdbc::io::chan_stream::{ChanStream, TryStream};
    use cdbc_sqlite::{Sqlite, SqliteRow};
    use crate::make_sqlite;
    #[test]
    fn test_stream_sqlite() -> cdbc::Result<()> {
        //first. create sqlite dir/file
        let pool = make_sqlite().unwrap();
        //next create table and query result
        let mut conn = pool.acquire()?;
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
        Ok(())
    }
}
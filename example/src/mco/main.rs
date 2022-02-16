use std::fs::File;
use mco::coroutine::sleep;
use std::time::Duration;
use mco::co;
use cdbc::Executor;
use cdbc_sqlite::{Sqlite, SqlitePool};
use mco::coroutine::Builder;
use cdbc::pool::PoolOptions;

fn main(){
    let pool = make_sqlite().unwrap();
    //spawn coroutines
    let copy_pool = pool.clone();
    co!(move ||{
      run_sqlite(copy_pool);
    });
    let copy_pool2 = pool.clone();
    co!(Builder::new().name("co1".to_string()),move ||{
      run_sqlite(copy_pool2);
    });
    sleep(Duration::from_secs(3));
}

fn run_sqlite(pool:SqlitePool) -> cdbc::Result<()> {
    println!("run on coroutine:{:?}",mco::coroutine::current().name());
    #[derive(Debug)]
    pub struct BizActivity {
        pub id: Option<String>,
        pub name: Option<String>,
        pub delete_flag: Option<i32>,
    }
    let data = cdbc::row_scans!(
        cdbc::query("select * from biz_activity where id = ?")
        .bind("1")
        .fetch_all(pool.clone())?,
        BizActivity{id:None,name:None,delete_flag:None})?;
    println!("{:?}", data);
    Ok(())
}

fn make_sqlite() -> cdbc::Result<SqlitePool> {
    //first. create sqlite dir/file
    std::fs::create_dir_all("target/db/");
    File::create("target/db/sqlite.db");
    //Concurrent reads and writes to SQLite limit set connection number to 1
    let pool = PoolOptions::<Sqlite>::new().max_connections(1).connect("sqlite://target/db/sqlite.db")?;
    let mut conn = pool.acquire()?;
    conn.execute("CREATE TABLE biz_activity(  id string, name string,age int, delete_flag int) ");
    conn.execute("INSERT INTO biz_activity (id,name,age,delete_flag) values (\"1\",\"1\",1,0)");
    Ok(pool)
}
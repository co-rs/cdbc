use std::fs::File;
use may::coroutine::sleep;
use std::time::Duration;
use may::go;
use cdbc::Executor;
use cdbc_sqlite::SqlitePool;

fn main(){
    let pool = make_sqlite().unwrap();
    //spawn coroutines
    let copy_pool = pool.clone();
    go!(move ||{
      run_sqlite(copy_pool);
    });
    let copy_pool2 = pool.clone();
    go!(move ||{
      sleep(Duration::from_secs(1));
      run_sqlite(copy_pool2);
    });
    sleep(Duration::from_secs(3));
}

fn run_sqlite(pool:SqlitePool) -> cdbc::Result<()> {
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
    //next create table and query result
    let pool = SqlitePool::connect("sqlite://target/db/sqlite.db")?;
    let mut conn = pool.acquire()?;
    conn.execute("CREATE TABLE biz_activity(  id string, name string,age int, delete_flag int) ");
    conn.execute("INSERT INTO biz_activity (id,name,age,delete_flag) values (\"1\",\"1\",1,0)");
    Ok(pool)
}
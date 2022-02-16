use std::cell::RefCell;
use std::fs::File;
use mco::defer;
use cdbc::connection::Connection;
use cdbc::Executor;
use cdbc_sqlite::SqlitePool;

fn main() -> cdbc::Result<()> {
    let pool = make_sqlite()?;
    let mut tx = RefCell::new(pool.begin()?);

    defer!(||{
       // Defer was able to guarantee that even the following code panic would commit
       if !tx.borrow().is_done(){
           tx.borrow_mut().commit();
           println!("----------tx committed-----------");
        }
    });

    // //change this to true,also the tx will be committed
    // if true{
    //     panic!("oh it is panic!");
    // }

    let r = tx.borrow_mut().execute("update biz_activity set name = '2'")?;
    println!("rows_affected: {}", r.rows_affected());
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
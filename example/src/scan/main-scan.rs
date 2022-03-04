use std::fs::File;
use cdbc::{Executor, query};
use cdbc::crud::{CRUD, Table};
use cdbc::database::Database;
use cdbc_sqlite::{Sqlite, SqlitePool};
use cdbc::Scan;
use cdbc::scan::Scan;

/// or use this example
/// #[derive(Debug,cdbc::ScanSqlite,cdbc::ScanMssql,cdbc::ScanMysql,cdbc::ScanPg)]
#[derive(Debug, cdbc::Scan)]
pub struct BizActivity {
    pub id: Option<String>,
    pub name: Option<String>,
    pub age: Option<i32>,
    pub delete_flag: Option<i32>,
}


impl Table for BizActivity {
    fn table() -> &'static str {
        "biz_activity"
    }

    fn columns() -> &'static [&'static str] {
        &["id", "name", "age", "delete_flag"]
    }
}

impl CRUD<BizActivity> for SqlitePool {
    fn inserts(&mut self, arg: Vec<BizActivity>) -> cdbc::Result<u64> where BizActivity: Sized {
        if arg.len() == 0 {
            return Ok(0);
        }
        let mut sql = format!("insert into {} ({}) values ", BizActivity::table(), BizActivity::columns_str());
        let mut value_num = 0;
        for x in &arg {
            sql.push_str("(");
            sql.push_str(&BizActivity::values_str("?"));
            sql.push_str(")");
            if value_num != 0 {
                sql.push_str(",");
            }
            value_num += 1;
        }
        sql.pop();
        let mut q = query(sql.as_str());
        for arg in arg {
            q = q.bind(arg.id)
                .bind(arg.name)
                .bind(arg.age)
                .bind(arg.delete_flag);
        }
        self.execute(q).map(|r| {
            r.rows_affected()
        })
    }

    fn updates(&mut self, arg: Vec<BizActivity>) -> cdbc::Result<u64> where BizActivity: Sized {
        todo!()
    }

    fn find(&mut self, arg: &str) -> cdbc::Result<Option<BizActivity>> where BizActivity: Sized {
        todo!()
    }

    fn finds(&mut self, arg: &str) -> cdbc::Result<Vec<BizActivity>> where BizActivity: Sized {
        todo!()
    }

    fn delete(&mut self, arg: &str) -> cdbc::Result<u64> where {
        todo!()
    }
}

fn main() -> cdbc::Result<()> {
    let pool = make_sqlite()?;

    let arg = BizActivity {
        id: Some("2".to_string()),
        name: Some("2".to_string()),
        age: Some(2),
        delete_flag: Some(1),
    };
    // BizActivity::insert(&pool,arg).unwrap();
    let r = pool.clone().insert(arg);
    println!("insert = {:?}", r);

    let data = query!("select * from biz_activity limit 1")
        .fetch_one(pool.clone())
        .scan();
    println!("{:?}", data);

    let data = query!("select * from biz_activity")
        .fetch_all(pool.clone())
        .scan();
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

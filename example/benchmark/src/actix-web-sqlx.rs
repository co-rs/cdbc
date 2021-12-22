#![allow(unused_must_use)]

#[macro_use]
extern crate lazy_static;
use std::sync::Arc;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use sqlx_core::pool::PoolOptions;
use sqlx_core::pool::Pool;
use sqlx_core::executor::Executor;
use sqlx_core::mysql::{MySql, MySqlConnectOptions, MySqlPool, MySqlRow, MySqlSslMode};
use sqlx_core::row::Row;

async fn make_pool() -> MySqlPool{
    let mut v:MySqlConnectOptions= "mysql://root:123456@127.0.0.1:3306/test".parse().unwrap();
    v=v.ssl_mode(MySqlSslMode::Disabled);//close ssl
    MySqlPool::connect_with(v).await.unwrap()
}

async fn index(pool: web::Data<Arc<MySqlPool>>) -> impl Responder {
    let mut conn=pool.acquire().await.unwrap();
    let row:MySqlRow = conn.fetch_one("select count(1) as count from biz_activity").await.unwrap();
    let count:i64=row.get("count");
    HttpResponse::Ok().body(count.to_string())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //async std only create pool.
    let pool = async_std::task::block_on(async{
        let pool=make_pool().await;
        pool.acquire().await.unwrap();
        pool
    });
    let pool=Arc::new(pool);
    //router
    HttpServer::new(move || {
        App::new()
            //add into actix-web data
            .data(pool.clone())
            .route("/", web::get().to(index))
    })
        .bind("0.0.0.0:8000")?
        .run()
        .await
}

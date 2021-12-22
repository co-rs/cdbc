
#[macro_use]
extern crate tokio;

use std::net::SocketAddr;
use std::sync::Arc;
use sqlx_core::mysql::{MySqlConnectOptions, MySqlPool, MySqlRow, MySqlSslMode};
use axum::extract::Extension;
use axum::AddExtensionLayer;
use axum::{
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};
use axum::routing::get;
use sqlx_core::executor::Executor;
use sqlx_core::row::Row;

//mysql driver url
pub const MYSQL_URL: &'static str = "mysql://root:123456@localhost:3306/test";

//handler
pub async fn handler(pool: Extension<Arc<MySqlPool>>) -> String {
    let mut conn=pool.acquire().await.unwrap();
    let row:MySqlRow = conn.fetch_one("select count(1) as count from biz_activity").await.unwrap();
    let count:i64=row.get("count");
    return count.to_string();
}

async fn make_pool() -> MySqlPool{
    let mut v:MySqlConnectOptions= "mysql://root:123456@127.0.0.1:3306/test".parse().unwrap();
    v=v.ssl_mode(MySqlSslMode::Disabled);//close ssl
    MySqlPool::connect_with(v).await.unwrap()
}

#[tokio::main]
async fn main() {
    let pool = Arc::new(make_pool().await);
    // build our application with a route
    let app = Router::new().route("/", get(handler))
        .layer(AddExtensionLayer::new(pool));
    // run it
    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    hyper::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
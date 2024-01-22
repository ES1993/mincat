use mincat::{
    extract::Query,
    http::{get, Router},
};
use serde::Deserialize;
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    // client url = /hello?id=2&name=lucy
    let router = Router::new().route(hello1).route(hello2);

    mincat::router(router).run("127.0.0.1:3000").await;
}

#[get("/hello1")]
async fn hello1(Query(query): Query<HashMap<String, String>>) -> &'static str {
    dbg!(query);
    "hello word"
}

#[derive(Debug, Clone, Deserialize)]
struct QueryParams {
    id: u64,
    name: String,
}

#[get("/hello2")]
async fn hello2(Query(query): Query<QueryParams>) -> &'static str {
    dbg!(&query, &query.id, &query.name);
    "hello word"
}

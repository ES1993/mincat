use mincat::{
    extract::form::FormUrlencoded,
    http::{post, Router},
};
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() {
    let router = Router::new().route(hello);

    mincat::router(router).run("127.0.0.1:3000").await;
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Data {
    id: u64,
    name: String,
}

#[post("/hello")]
async fn hello(FormUrlencoded(data): FormUrlencoded<Data>) -> FormUrlencoded<Data> {
    dbg!(&data);
    FormUrlencoded(data)
}

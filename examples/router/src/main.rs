use mincat::http::{get, Router};

#[tokio::main]
async fn main() {
    let router1 = Router::new().route(hello1);

    let router2 = Router::new().route(hello2);

    let router3 = Router::new().group("/api", Router::new().route(hello3));

    let router = Router::new().merge(router1).merge(router2).merge(router3);

    mincat::router(router).run("127.0.0.1:3000").await;
}

#[get("/hello1")]
async fn hello1() -> &'static str {
    "hello word1"
}

#[get("/hello2")]
async fn hello2() -> &'static str {
    "hello word2"
}

#[get("/hello3")]
async fn hello3() -> &'static str {
    "hello word3"
}

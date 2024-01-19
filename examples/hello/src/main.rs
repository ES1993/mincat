use mincat::{http::get, router::Router};

#[tokio::main]
async fn main() {
    let router = Router::new().route(hello);

    mincat::router(router).state().run("127.0.0.1:3000").await;
}

#[get("/hello")]
async fn hello() -> &'static str {
    "hello word"
}

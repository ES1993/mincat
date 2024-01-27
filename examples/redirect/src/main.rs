use mincat::{
    http::{get, Router},
    response::redirect::Redirect,
};

#[tokio::main]
async fn main() {
    let router = Router::new().route(hello1).route(hello2);

    mincat::router(router).run("127.0.0.1:3000").await;
}

#[get("/hello1")]
async fn hello1() -> Redirect {
    Redirect::sse("/hello2")
}

#[get("/hello2")]
async fn hello2() -> &'static str {
    "hello word 2"
}

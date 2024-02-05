use mincat::{
    http::{get, Router, StatusCode},
    middleware::HttpLog,
};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .with_target(false)
        .init();

    let router = Router::new().route(hello).middleware(HttpLog);

    mincat::router(router).run("127.0.0.1:3000").await;
}

#[get("/hello")]
async fn hello() -> Result<&'static str, (StatusCode, &'static str)> {
    Err((StatusCode::BAD_REQUEST, "some error"))
}

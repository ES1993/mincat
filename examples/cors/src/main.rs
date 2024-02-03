use mincat::{
    http::{get, Router},
    middleware::cors::{Any, Cors},
};

#[tokio::main]
async fn main() {
    let router = Router::new().route(hello).middleware(
        Cors::new()
            .allow_headers(Any)
            .allow_methods(Any)
            .allow_origin(Any)
            .allow_credentials(false),
    );

    mincat::router(router).run("127.0.0.1:3000").await;
}

#[get("/hello")]
async fn hello() -> &'static str {
    "hello word"
}

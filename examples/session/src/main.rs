use mincat::{
    extract::Session,
    http::{get, Router},
    middleware::session::{MemorySession, StoreSession},
};

#[tokio::main]
async fn main() {
    let router = Router::new()
        .route(hello)
        .middleware(StoreSession::from(MemorySession));

    mincat::router(router).run("127.0.0.1:3000").await;
}

#[get("/hello")]
async fn hello(session: Session) -> &'static str {
    "hello word"
}

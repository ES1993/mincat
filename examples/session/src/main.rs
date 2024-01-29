use mincat::{
    extract::{cookie::CookieKey, Session},
    http::{get, Router},
    middleware::session::{MemorySession, StoreSession},
};

#[tokio::main]
async fn main() {
    let router = Router::new()
        .route(hello)
        .middleware(StoreSession::from(MemorySession::default()));

    mincat::router(router)
        .state(CookieKey::from("xxxx"))
        .run("127.0.0.1:3000")
        .await;
}

#[get("/hello")]
async fn hello(mut session: Session) -> &'static str {
    let user_name = session.get::<String>("user name").await.unwrap();
    dbg!(user_name);
    session.set("user name", "xiao li").await.unwrap();

    let count = session
        .get::<usize>("user name count")
        .await
        .unwrap()
        .unwrap_or(0);
    dbg!(count);
    session.set("user name count", count + 1).await.unwrap();

    "hello word"
}

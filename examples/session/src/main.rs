use mincat::{
    extract::{cookie::CookieKey, Session},
    http::{get, Router},
    middleware::session::{MemorySessionBuilder, StoreSession},
};

#[tokio::main]
async fn main() {
    let memory_session = MemorySessionBuilder::default()
        .age(10) // Expiration time (unit seconds,default 3600)
        .interval(30) // Cleanup expired task cycle time (unit seconds,default 60)
        .build()
        .unwrap();
    let stor_session = StoreSession::from(memory_session);

    let router = Router::new().route(hello).middleware(stor_session);

    mincat::router(router)
        .state(CookieKey::from("xxxx"))
        .run("127.0.0.1:3000")
        .await;
}

#[get("/hello")]
async fn hello(session: Session) -> &'static str {
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

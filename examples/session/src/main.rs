use mincat::{
    extract::{cookie::CookieKey, Session},
    http::{get, Router},
    middleware::session::{
        MemorySessionBuilder, MysqlSessionBuilder, PostgresSessionBuilder,
        RedisClusterSessionBuilder, RedisSessionBuilder, StoreSession,
    },
};

#[allow(unused_variables)]
#[tokio::main]
async fn main() {
    let memory_session = MemorySessionBuilder::default()
        .age(20) // Expiration time (unit seconds,default 3600)
        .interval(30) // Cleanup expired task cycle time (unit seconds,default 60)
        .build()
        .unwrap();

    let redis_session = RedisSessionBuilder::default()
        .age(20)
        .url("redis://:bitnami@localhost:16381/0".to_string())
        .prefix("session_key".to_string())
        .build()
        .unwrap();

    let redis_cluster_session = RedisClusterSessionBuilder::default()
        .age(20)
        .urls(vec![
            "redis://:bitnami@localhost:16381/0".to_string(),
            "redis://:bitnami@localhost:16382/0".to_string(),
            "redis://:bitnami@localhost:16383/0".to_string(),
        ])
        .prefix("session_key".to_string())
        .build()
        .unwrap();

    let postgres_session = PostgresSessionBuilder::default()
        .url("postgresql://localhost:30000/postgres".to_string())
        .table_name("xxx_session".to_string())
        .age(20)
        .interval(30)
        .build()
        .unwrap();

    let mysql_session = MysqlSessionBuilder::default()
        .url("mysql://localhost:3306/sys".to_string())
        .table_name("xxx_session".to_string())
        .age(20)
        .interval(30)
        .build()
        .unwrap();

    // let stor_session = StoreSession::from(memory_session);
    // let stor_session = StoreSession::from(redis_session);
    // let stor_session = StoreSession::from(redis_cluster_session);
    // let stor_session = StoreSession::from(postgres_session);
    let stor_session = StoreSession::from(mysql_session);

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

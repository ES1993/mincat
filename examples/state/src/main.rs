use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, RwLock,
};

use mincat::{extract::State, http::get, router::Router};

#[derive(Debug, Clone, Default)]
struct App {
    count_normal: usize,
    count_atomic: Arc<AtomicUsize>,
    count_lock: Arc<RwLock<usize>>,
}

#[tokio::main]
async fn main() {
    let router = Router::new().route(hello);

    mincat::router(router)
        .state(App::default())
        .run("127.0.0.1:3000")
        .await;
}

#[get("/hello")]
async fn hello(State(mut app): State<App>) -> &'static str {
    app.count_normal += 1;
    app.count_atomic.fetch_add(1, Ordering::SeqCst);
    *app.count_lock.write().unwrap() += 1;
    dbg!(app);
    "hello word"
}

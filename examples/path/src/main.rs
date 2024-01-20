use std::collections::HashMap;

use mincat::{extract::Path, http::get, router::Router};

#[tokio::main]
async fn main() {
    let router = Router::new().route(hello1).route(hello2).route(hello3);

    mincat::router(router).run("127.0.0.1:3000").await;
}

#[get("/hello1/:id/:name")]
async fn hello1(Path((id, name)): Path<(usize, String)>) -> &'static str {
    dbg!(id, name);
    "hello word"
}

#[get("/hello2/:id/:name")]
async fn hello2(Path(path): Path<HashMap<String, String>>) -> &'static str {
    dbg!(path);
    "hello word"
}

#[derive(Debug, Clone, serde::Deserialize)]
struct PathParams {
    id: u64,
    name: String,
}

#[get("/hello3/:id/:name")]
async fn hello3(Path(path): Path<PathParams>) -> &'static str {
    dbg!(&path, &path.id, &path.name);
    "hello word"
}

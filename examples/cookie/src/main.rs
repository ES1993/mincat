use std::collections::HashMap;

use mincat::{
    extract::{
        cookie::{Cookie, CookieJar, CookieKey, PrivateCookieJar, SignedCookieJar},
        Path,
    },
    http::{get, Router},
    response::redirect::Redirect,
};

#[tokio::main]
async fn main() {
    let router = Router::new()
        .route(hello1)
        .route(hello2)
        .route(hello3)
        .route(hello4);

    mincat::router(router)
        .state(CookieKey::from("xxxxxxxx"))
        .run("127.0.0.1:3000")
        .await;
}

#[get("/hello1")]
async fn hello1(cookie: CookieJar) -> (CookieJar, Redirect) {
    dbg!(cookie.get("hello1"));
    (
        cookie.add(Cookie::build(("hello1", "hello1")).http_only(true)),
        Redirect::sse_other("/hello4/hello1"),
    )
}

#[get("/hello2")]
async fn hello2(cookie: PrivateCookieJar) -> (PrivateCookieJar, Redirect) {
    dbg!(cookie.get("hello2"));
    (
        cookie.add(Cookie::build(("hello2", "hello2")).http_only(true)),
        Redirect::sse_other("/hello4/hello2"),
    )
}

#[get("/hello3")]
async fn hello3(cookie: SignedCookieJar) -> (SignedCookieJar, Redirect) {
    dbg!(cookie.get("hello3"));
    (
        cookie.add(Cookie::build(("hello3", "hello3")).http_only(true)),
        Redirect::sse_other("/hello4/hello3"),
    )
}

#[get("/hello4/:cookie")]
async fn hello4(Path(path): Path<HashMap<String, String>>) -> String {
    if let Some(path) = path.get("cookie") {
        return path.to_owned();
    }
    "first".to_string()
}

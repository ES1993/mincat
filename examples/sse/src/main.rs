use futures::stream::{self};
use mincat::{
    http::{get, Router},
    response::{Event, KeepAlive, Sse},
};
use std::time::Duration;
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() {
    let router = Router::new().route(hello);

    mincat::router(router).run("127.0.0.1:3000").await;
}

#[get("/hello")]
async fn hello() -> Sse {
    let stream = stream::repeat_with(|| Event::default().event("hello word!"))
        .throttle(Duration::from_secs(1));

    Sse::new(stream).keep_alive(KeepAlive::default())
}

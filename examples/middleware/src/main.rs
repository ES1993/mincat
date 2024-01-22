use mincat::{
    http::{get, Request, Response, Router},
    middleware::{middleware, Next},
};

#[tokio::main]
async fn main() {
    let router = Router::new()
        .route(hello.middleware(middleware1).middleware(middleware2))
        .middleware(middleware3)
        .middleware(middleware4);

    let router = Router::new()
        .group("/api", router)
        .route(hello)
        .middleware(middleware5);

    let router = Router::new()
        .merge(router)
        .route(hello)
        .middleware(middleware6);

    mincat::router(router).run("127.0.0.1:3000").await;
}

#[get("/hello")]
async fn hello() -> &'static str {
    dbg!("hello");
    "hello word"
}

#[middleware]
async fn middleware1(request: Request, next: Next) -> Response {
    dbg!("middleware1 start");
    let response = next.run(request).await;
    dbg!("middleware1 end");
    response
}

#[middleware]
async fn middleware2(request: Request, next: Next) -> Response {
    dbg!("middleware2 start");
    let response = next.run(request).await;
    dbg!("middleware2 end");
    response
}

#[middleware]
async fn middleware3(request: Request, next: Next) -> Response {
    dbg!("middleware3 start");
    let response = next.run(request).await;
    dbg!("middleware3 end");
    response
}

#[middleware]
async fn middleware4(request: Request, next: Next) -> Response {
    dbg!("middleware4 start");
    let response = next.run(request).await;
    dbg!("middleware4 end");
    response
}

#[middleware]
async fn middleware5(request: Request, next: Next) -> Response {
    dbg!("middleware5 start");
    let response = next.run(request).await;
    dbg!("middleware5 end");
    response
}

#[middleware]
async fn middleware6(request: Request, next: Next) -> Response {
    dbg!("middleware6 start");
    let response = next.run(request).await;
    dbg!("middleware6 end");
    response
}

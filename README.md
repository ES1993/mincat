# mincat

mincat is a fast and compact server-side framework

```rust
use mincat::http::{get, Router};

#[tokio::main]
async fn main() {
    let router = Router::new().route(hello);

    mincat::router(router).run("127.0.0.1:3000").await;
}

#[get("/hello")]
async fn hello() -> &'static str {
    "hello word"
}
```

English · [简体中文](./README.zh-cn.md)

# 例子

1. [Basic Hello World](./examples/hello/src/main.rs)
2. [How to Limit Body Size](./examples/body-limit/src/main.rs)
3. [How to Use Cookies](./examples/cookie/src/main.rs)
4. [How to Set Up Cross-Origin Resource Sharing (CORS)](./examples/cors/src/main.rs)
5. [How to Customize Errors](./examples/custom-error/src/main.rs)
6. [How to Customize Extractors for Parameters](./examples/custom-extract/src/main.rs)
7. [How to Customize Responses](./examples/custom-response/src/main.rs)
8. [How to Extract FormData](./examples/form-data/src/main.rs)
9. [How to Extract FormUrlencoded Data](./examples/form-urlencoded/src/main.rs)
10. [How to Use Logging](./examples/http-log/src/main.rs)
11. [How to Extract JSON](./examples/json/src/main.rs)
12. [How to Customize Middleware](./examples/middleware/src/main.rs)
13. [How to Extract URL Path Parameters](./examples/path/src/main.rs)
14. [How to Extract URL Query Parameters](./examples/query/src/main.rs)
15. [How to Redirect](./examples/redirect/src/main.rs)
16. [How to Use Routing](./examples/router/src/main.rs)
17. [How to Use Sessions](./examples/session/src/main.rs)
18. [How to Use Server-Sent Events (SSE)](./examples/sse/src/main.rs)
19. [How to Use Global State](./examples/state/src/main.rs)
20. [How to Proxy Static Folders](./examples/static-file/src/main.rs)
21. [How to Upload Files](./examples/upload-file/src/main.rs)
22. [How to Use WebSockets](./examples/websocket/src/main.rs)

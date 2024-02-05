# MinCat

mincat是一个快速小巧的服务端框架

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

[English](./README.md) · 简体中文

# 安装

```shell
cargo add mincat
```

# 例子

1. [基本的hello word](./examples/hello/src/main.rs)
2. [如何限制body大小](./examples/body-limit/src/main.rs)
3. [如何使用cookie](./examples/cookie/src/main.rs)
4. [如何设置跨越（cors）](./examples/cors/src/main.rs)
5. [如何自定义错误](./examples/custom-error/src/main.rs)
6. [如何自定义参数提取器](./examples/custom-extract/src/main.rs)
7. [如何自定义响应](./examples/custom-response/src/main.rs)
8. [如何提取FormData数据](./examples/form-data/src/main.rs)
9. [如何提取FormUrlencoded数据](./examples/form-urlencoded/src/main.rs)
10. [如何使用日志](./examples/http-log/src/main.rs)
11. [如何提取json](./examples/json/src/main.rs)
12. [如何自定义中间件](./examples/middleware/src/main.rs)
13. [如何提取url path 参数](./examples/path/src/main.rs)
14. [如何提取url query 参数](./examples/query/src/main.rs)
15. [如何重定向](./examples/redirect/src/main.rs)
16. [如何使用路由](./examples/router/src/main.rs)
17. [如何使用session](./examples/session/src/main.rs)
18. [如何使用sse](./examples/sse/src/main.rs)
19. [如何使用全局状态](./examples/state/src/main.rs)
20. [如何代理静态文件夹](./examples/static-file/src/main.rs)
21. [如何上传文件](./examples/upload-file/src/main.rs)
22. [如何使用websocket](./examples/websocket/src/main.rs)

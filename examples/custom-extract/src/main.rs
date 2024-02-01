use mincat::http::{get, FromRequestParts, Parts, Router, StatusCode};

#[tokio::main]
async fn main() {
    let router = Router::new().route(hello);

    mincat::router(router).run("127.0.0.1:3000").await;
}

#[get("/hello")]
async fn hello(HeaderUserName(user_name): HeaderUserName) -> &'static str {
    dbg!(user_name);
    "hello word"
}

struct HeaderUserName(String);

#[async_trait::async_trait]
impl FromRequestParts for HeaderUserName {
    type Error = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts) -> Result<Self, Self::Error> {
        let user_name = parts
            .headers
            .get("user-name")
            .ok_or((StatusCode::BAD_REQUEST, "header missing field `user-name`"))?
            .to_str()
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "header field `user-name` can't convert to str",
                )
            })?
            .to_string();

        Ok(HeaderUserName(user_name))
    }
}

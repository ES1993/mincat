use mincat::http::{
    get,
    header::{self, HeaderValue},
    mime, IntoResponse, Response, Router, StatusCode,
};

#[tokio::main]
async fn main() {
    let router = Router::new().route(hello);

    mincat::router(router).run("127.0.0.1:3000").await;
}

#[get("/hello")]
async fn hello() -> Text {
    Text("hello word".to_string())
}

struct Text(String);

impl IntoResponse for Text {
    fn into_response(self) -> Response {
        (
            StatusCode::OK,
            [(
                header::CONTENT_TYPE,
                HeaderValue::from_static(mime::TEXT.as_ref()),
            )],
            self.0,
        )
            .into_response()
    }
}

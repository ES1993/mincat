use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use mincat::{
    extract::State,
    http::{
        get,
        header::{self, HeaderValue},
        mime, IntoResponse, Response, Router, StatusCode,
    },
};

#[tokio::main]
async fn main() {
    let router = Router::new().route(hello);

    mincat::router(router)
        .state(AppState::default())
        .run("127.0.0.1:3000")
        .await;
}

#[get("/hello")]
async fn hello(State(app_state): State<AppState>) -> Result<&'static str, AppError> {
    app_state.count.fetch_add(1, Ordering::SeqCst);
    let count = app_state.count.load(Ordering::SeqCst);

    if count == 1 {
        Err(AppError::StringError("test string error".to_string()))
    } else if count == 2 {
        Err(AppError::StrError("test str error"))
    } else if count % 5 == 0 {
        Err(AppError::Unkonw)
    } else {
        Ok("hello word")
    }
}

#[derive(Default, Clone)]
struct AppState {
    count: Arc<AtomicUsize>,
}

#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error("StringError: {0}")]
    StringError(String),
    #[error("StrError: {0}")]
    StrError(&'static str),
    #[error("Unkonw")]
    Unkonw,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::BAD_REQUEST,
            [(
                header::CONTENT_TYPE,
                HeaderValue::from_static(mime::TEXT.as_ref()),
            )],
            self.to_string(),
        )
            .into_response()
    }
}

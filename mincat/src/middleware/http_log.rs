use mincat_core::{middleware::Middleware, next::Next, request::Request, response::Response};
use std::time::Instant;
use tracing::{event, Level};

#[derive(Debug)]
pub struct HttpLog;

#[async_trait::async_trait]
impl Middleware for HttpLog {
    #[tracing::instrument(
        name = "mincat[http-log]", 
        skip(self, request, next), 
        fields(
            uri = request.uri().to_string(), 
            method = request.method().to_string(),
        )
    )]
    async fn call(self: Box<Self>, request: Request, next: Next) -> Response {
        let start = Instant::now();
        event!(Level::INFO, "REQUEST");
        let res = next.run(request).await;
        let code = u16::from(res.status());
        let duration = start.elapsed();
        if code == 200 {
            event!(Level::INFO, "RESPONSE CODE:{code} TIME:{:?}]", duration);
        } else {
            event!(Level::ERROR, "RESPONSE CODE:{code}");
        }
        res
    }

    fn clone_box(&self) -> Box<dyn Middleware> {
        Box::new(HttpLog)
    }
}

impl From<HttpLog> for Box<dyn Middleware> {
    fn from(value: HttpLog) -> Box<dyn Middleware> {
        value.clone_box()
    }
}

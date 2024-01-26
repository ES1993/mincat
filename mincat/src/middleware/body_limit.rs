use mincat_core::{
    body::BodyLimitedSize, middleware::Middleware, next::Next, request::Request, response::Response,
};

pub struct BodyLimit(pub usize);

#[async_trait::async_trait]
impl Middleware for BodyLimit {
    async fn call(self: Box<Self>, mut request: Request, next: Next) -> Response {
        request.extensions_mut().insert(BodyLimitedSize(self.0));
        next.run(request).await
    }

    fn clone_box(&self) -> Box<dyn Middleware> {
        Box::new(BodyLimit(self.0))
    }
}

impl From<BodyLimit> for Box<dyn Middleware> {
    fn from(value: BodyLimit) -> Box<dyn Middleware> {
        value.clone_box()
    }
}

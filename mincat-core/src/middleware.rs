use crate::{request::Request, response::Response};

#[async_trait::async_trait]
pub trait Middleware: Send + Sync + 'static {
    async fn on_request(&self, request: &mut Request, response: &mut Response);
    async fn on_response(&self, request: &mut Request, response: &mut Response);
    fn clone_box(&self) -> Box<dyn Middleware>;
}

impl Clone for Box<dyn Middleware> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

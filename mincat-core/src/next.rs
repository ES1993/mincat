use crate::{handler::Handler, request::Request, response::Response};

#[derive(Clone)]
pub struct Next(Handler);

impl Next {
    pub fn new(handler: Handler) -> Self {
        Self(handler)
    }

    pub async fn run(mut self, request: Request) -> Response {
        if let Some(arr) = &mut self.0.middleware {
            if let Some(middleware) = arr.pop_back() {
                middleware.call(request, self).await
            } else {
                self.0.func.call(request).await
            }
        } else {
            self.0.func.call(request).await
        }
    }
}

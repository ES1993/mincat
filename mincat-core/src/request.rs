use crate::{body::Body, response::IntoResponse};

pub type Request<T = Body> = http::Request<T>;

#[async_trait::async_trait]
pub trait FromRequest: Clone + Send + 'static {
    type Error: IntoResponse;

    async fn from_request(request: &mut Request) -> Result<Self, Self::Error>;
}

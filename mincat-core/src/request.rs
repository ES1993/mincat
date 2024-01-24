use crate::{
    body::{Body, BodyLimitedSize},
    response::IntoResponse,
};

pub use http::request::Parts;
pub use http_body_util::Limited;

pub type Request<T = Body> = http::Request<T>;

pub trait RequestExt {
    fn change_to_limited_body(self) -> Self;
}

impl RequestExt for Request {
    fn change_to_limited_body(self) -> Self {
        match self.extensions().get::<BodyLimitedSize>().copied() {
            Some(BodyLimitedSize(size)) => self.map(|body| Body::new(Limited::new(body, size))),
            None => self,
        }
    }
}

#[async_trait::async_trait]
pub trait FromRequest: Sized {
    type Error: IntoResponse;

    async fn from_request(request: Request) -> Result<Self, Self::Error>;
}

#[async_trait::async_trait]
impl<T> FromRequest for T
where
    T: FromRequestParts,
{
    type Error = <Self as FromRequestParts>::Error;

    async fn from_request(req: Request) -> Result<Self, Self::Error> {
        let (mut parts, _) = req.into_parts();
        Self::from_request_parts(&mut parts).await
    }
}

#[async_trait::async_trait]
pub trait FromRequestParts: Sized {
    type Error: IntoResponse;

    async fn from_request_parts(parts: &mut Parts) -> Result<Self, Self::Error>;
}

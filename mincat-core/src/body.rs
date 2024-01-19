use std::error::Error;

use bytes::Bytes;
use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use hyper::body::Incoming;

type BoxBodyError = Box<dyn Error + Send + Sync>;
pub struct Body(BoxBody<Bytes, BoxBodyError>);

impl Body {
    pub fn incoming(incoming: Incoming) -> Self {
        Body(incoming.boxed().map_err(Into::into).boxed())
    }

    pub fn box_body(body: Body) -> BoxBody<Bytes, BoxBodyError> {
        body.0
    }

    pub fn empty() -> Self {
        Body(Empty::new().map_err(Into::into).boxed())
    }
}

impl From<&'static str> for Body {
    fn from(value: &'static str) -> Self {
        Body(full(value))
    }
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, BoxBodyError> {
    Full::new(chunk.into()).map_err(Into::into).boxed()
}

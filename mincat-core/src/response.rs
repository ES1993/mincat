use std::convert::Infallible;

use http::{header, HeaderValue};

use crate::body::Body;

pub type Response<T = Body> = http::Response<T>;

pub trait IntoResponse {
    fn into_response(self) -> Response;
}

impl IntoResponse for () {
    fn into_response(self) -> Response {
        Response::new(Body::empty())
    }
}

impl IntoResponse for Infallible {
    fn into_response(self) -> Response {
        Response::new(Body::empty())
    }
}

impl IntoResponse for &'static str {
    fn into_response(self) -> Response {
        let mut res = Response::new(Body::from(self));
        res.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_PLAIN_UTF_8.as_ref()),
        );
        res
    }
}

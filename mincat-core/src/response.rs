use std::convert::Infallible;

use bytes::Bytes;
use http::{header, HeaderName, HeaderValue, StatusCode};

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

impl<T, E> IntoResponse for Result<T, E>
where
    T: IntoResponse,
    E: IntoResponse,
{
    fn into_response(self) -> Response {
        match self {
            Ok(value) => value.into_response(),
            Err(err) => err.into_response(),
        }
    }
}

impl IntoResponse for Body {
    fn into_response(self) -> Response {
        Response::new(self)
    }
}

impl IntoResponse for StatusCode {
    fn into_response(self) -> Response {
        let mut res = ().into_response();
        *res.status_mut() = self;
        res
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

impl IntoResponse for String {
    fn into_response(self) -> Response {
        let mut res = Response::new(Body::from(self));
        res.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_PLAIN_UTF_8.as_ref()),
        );
        res
    }
}

impl IntoResponse for Response {
    fn into_response(self) -> Response {
        self
    }
}

impl IntoResponse for Bytes {
    fn into_response(self) -> Response {
        let mut res = Body::from(self).into_response();
        res.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::APPLICATION_OCTET_STREAM.as_ref()),
        );
        res
    }
}

impl<R> IntoResponse for (StatusCode, R)
where
    R: IntoResponse,
{
    fn into_response(self) -> Response {
        let mut res = self.1.into_response();
        *res.status_mut() = self.0;
        res
    }
}

fn insert_headers<K, V, const N: usize>(
    mut res: Response,
    arr: [(K, V); N],
) -> Result<Response, String>
where
    K: TryInto<HeaderName>,
    K::Error: std::fmt::Display,
    V: TryInto<HeaderValue>,
    V::Error: std::fmt::Display,
{
    for (key, value) in arr {
        let key = key.try_into().map_err(|e| e.to_string())?;
        let value = value.try_into().map_err(|e| e.to_string())?;
        res.headers_mut().insert(key, value);
    }
    Ok(res)
}

impl<K, V, const N: usize, R> IntoResponse for (StatusCode, [(K, V); N], R)
where
    R: IntoResponse,
    K: TryInto<HeaderName>,
    K::Error: std::fmt::Display,
    V: TryInto<HeaderValue>,
    V::Error: std::fmt::Display,
{
    fn into_response(self) -> Response {
        let mut res = self.2.into_response();
        *res.status_mut() = self.0;
        insert_headers(res, self.1).into_response()
    }
}

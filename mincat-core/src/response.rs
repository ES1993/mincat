use std::convert::Infallible;

use bytes::Bytes;
use http::{header, HeaderName, HeaderValue, StatusCode};
use mincat_macro::repeat_macro_max_generics_param;

use crate::body::Body;

pub type Response<T = Body> = http::Response<T>;

pub trait IntoResponse {
    fn into_response(self) -> Response;
}

pub trait IntoResponseParts {
    fn into_response_parts(self, response: Response) -> Response;
}

impl IntoResponse for () {
    fn into_response(self) -> Response {
        Response::new(Body::empty())
    }
}

impl IntoResponseParts for () {
    fn into_response_parts(self, response: Response) -> Response {
        response
    }
}

impl IntoResponse for Infallible {
    fn into_response(self) -> Response {
        Response::new(Body::empty())
    }
}

impl IntoResponseParts for Infallible {
    fn into_response_parts(self, response: Response) -> Response {
        response
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

impl IntoResponseParts for StatusCode {
    fn into_response_parts(self, mut response: Response) -> Response {
        *response.status_mut() = self;
        response
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

impl<K, V, const N: usize> IntoResponse for [(K, V); N]
where
    K: TryInto<HeaderName>,
    K::Error: std::fmt::Display,
    V: TryInto<HeaderValue>,
    V::Error: std::fmt::Display,
{
    fn into_response(self) -> Response {
        let res = ().into_response();
        insert_headers(res, self).into_response()
    }
}

impl<K, V, const N: usize> IntoResponseParts for [(K, V); N]
where
    K: TryInto<HeaderName>,
    K::Error: std::fmt::Display,
    V: TryInto<HeaderValue>,
    V::Error: std::fmt::Display,
{
    fn into_response_parts(self, response: Response) -> Response {
        insert_headers(response, self).into_response()
    }
}

macro_rules! impl_into_response_parts_tuple {
    ( $($ty:ident),* ) => {
        #[allow(non_snake_case)]
        impl<$($ty,)*> IntoResponseParts for ($($ty,)*)
        where
            $( $ty: IntoResponseParts, )*
        {
            fn into_response_parts(self,response: Response) -> Response {
                let ($($ty,)*) = self;
                $(
                    let response = $ty.into_response_parts(response);
                )*
                response
            }
        }
    }
}

repeat_macro_max_generics_param!(impl_into_response_parts_tuple, 1, 17, P);

macro_rules! impl_into_response_tuple {
    ( $($ty:ident),* ) => {
        #[allow(non_snake_case)]
        impl<R, $($ty,)*> IntoResponse for ($($ty),*, R)
        where
            $( $ty: IntoResponseParts, )*
            R: IntoResponse,
        {
            fn into_response(self) -> Response {
                let ( $($ty),*, response) = self;
                let response = response.into_response();
                $(
                    let response = $ty.into_response_parts(response);
                )*
                response
            }
        }
    };
}

repeat_macro_max_generics_param!(impl_into_response_tuple, 1, 17, P);

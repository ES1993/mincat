use bytes::{BufMut, BytesMut};
use http::{
    header::{self, HeaderName},
    HeaderMap, HeaderValue, Method, StatusCode,
};
use mincat_core::{
    middleware::Middleware,
    next::Next,
    request::Request,
    response::{IntoResponse, Response},
};

mod allow_credentials;
mod allow_headers;
mod allow_methods;
mod allow_origin;
mod allow_private_network;
mod expose_headers;
mod max_age;
mod vary;

pub use self::{
    allow_credentials::AllowCredentials, allow_headers::AllowHeaders, allow_methods::AllowMethods,
    allow_origin::AllowOrigin, allow_private_network::AllowPrivateNetwork,
    expose_headers::ExposeHeaders, max_age::MaxAge, vary::Vary,
};

#[allow(clippy::declare_interior_mutable_const)]
const WILDCARD: HeaderValue = HeaderValue::from_static("*");

fn separated_by_commas<I>(mut iter: I) -> Option<HeaderValue>
where
    I: Iterator<Item = HeaderValue>,
{
    match iter.next() {
        Some(fst) => {
            let mut result = BytesMut::from(fst.as_bytes());
            for val in iter {
                result.reserve(val.len() + 1);
                result.put_u8(b',');
                result.extend_from_slice(val.as_bytes());
            }

            Some(HeaderValue::from_maybe_shared(result.freeze()).unwrap())
        }
        None => None,
    }
}

fn preflight_request_headers() -> impl Iterator<Item = HeaderName> {
    IntoIterator::into_iter([
        header::ORIGIN,
        header::ACCESS_CONTROL_REQUEST_METHOD,
        header::ACCESS_CONTROL_REQUEST_HEADERS,
    ])
}

fn ensure_usable_cors_rules(cors: &Cors) {
    if cors.allow_credentials.is_true() {
        assert!(
            !cors.allow_headers.is_wildcard(),
            "Invalid CORS configuration: Cannot combine `Access-Control-Allow-Credentials: true` \
             with `Access-Control-Allow-Headers: *`"
        );

        assert!(
            !cors.allow_methods.is_wildcard(),
            "Invalid CORS configuration: Cannot combine `Access-Control-Allow-Credentials: true` \
             with `Access-Control-Allow-Methods: *`"
        );

        assert!(
            !cors.allow_origin.is_wildcard(),
            "Invalid CORS configuration: Cannot combine `Access-Control-Allow-Credentials: true` \
             with `Access-Control-Allow-Origin: *`"
        );

        assert!(
            !cors.expose_headers.is_wildcard(),
            "Invalid CORS configuration: Cannot combine `Access-Control-Allow-Credentials: true` \
             with `Access-Control-Expose-Headers: *`"
        );
    }
}

#[derive(Debug, Clone, Copy)]
#[must_use]
pub struct Any;

#[derive(Debug, Clone)]
pub struct Cors {
    allow_credentials: AllowCredentials,
    allow_headers: AllowHeaders,
    allow_methods: AllowMethods,
    allow_origin: AllowOrigin,
    allow_private_network: AllowPrivateNetwork,
    expose_headers: ExposeHeaders,
    max_age: MaxAge,
    vary: Vary,
}

impl Cors {
    pub fn new() -> Self {
        Self {
            allow_credentials: Default::default(),
            allow_headers: Default::default(),
            allow_methods: Default::default(),
            allow_origin: Default::default(),
            allow_private_network: Default::default(),
            expose_headers: Default::default(),
            max_age: Default::default(),
            vary: Default::default(),
        }
    }

    pub fn permissive() -> Self {
        Self::new()
            .allow_headers(Any)
            .allow_methods(Any)
            .allow_origin(Any)
            .expose_headers(Any)
    }

    pub fn very_permissive() -> Self {
        Self::new()
            .allow_credentials(true)
            .allow_headers(AllowHeaders::mirror_request())
            .allow_methods(AllowMethods::mirror_request())
            .allow_origin(AllowOrigin::mirror_request())
    }

    pub fn allow_credentials<T>(mut self, allow_credentials: T) -> Self
    where
        T: Into<AllowCredentials>,
    {
        self.allow_credentials = allow_credentials.into();
        self
    }

    pub fn allow_headers<T>(mut self, headers: T) -> Self
    where
        T: Into<AllowHeaders>,
    {
        self.allow_headers = headers.into();
        self
    }

    pub fn max_age<T>(mut self, max_age: T) -> Self
    where
        T: Into<MaxAge>,
    {
        self.max_age = max_age.into();
        self
    }

    pub fn allow_methods<T>(mut self, methods: T) -> Self
    where
        T: Into<AllowMethods>,
    {
        self.allow_methods = methods.into();
        self
    }

    pub fn allow_origin<T>(mut self, origin: T) -> Self
    where
        T: Into<AllowOrigin>,
    {
        self.allow_origin = origin.into();
        self
    }

    pub fn expose_headers<T>(mut self, headers: T) -> Self
    where
        T: Into<ExposeHeaders>,
    {
        self.expose_headers = headers.into();
        self
    }

    pub fn allow_private_network<T>(mut self, allow_private_network: T) -> Self
    where
        T: Into<AllowPrivateNetwork>,
    {
        self.allow_private_network = allow_private_network.into();
        self
    }

    pub fn vary<T>(mut self, headers: T) -> Self
    where
        T: Into<Vary>,
    {
        self.vary = headers.into();
        self
    }
}

impl Default for Cors {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Middleware for Cors {
    async fn call(self: Box<Self>, request: Request, next: Next) -> Response {
        ensure_usable_cors_rules(&self);
        let (parts, body) = request.into_parts();
        let origin = parts.headers.get(&header::ORIGIN);
        let mut headers = HeaderMap::new();
        headers.extend(self.allow_origin.to_header(origin, &parts));
        headers.extend(self.allow_credentials.to_header(origin, &parts));
        headers.extend(self.allow_private_network.to_header(origin, &parts));
        headers.extend(self.vary.to_header());

        if parts.method == Method::OPTIONS {
            headers.extend(self.allow_methods.to_header(&parts));
            headers.extend(self.allow_headers.to_header(&parts));
            headers.extend(self.max_age.to_header(origin, &parts));

            (StatusCode::OK, headers, ()).into_response()
        } else {
            headers.extend(self.expose_headers.to_header(&parts));
            let request = Request::from_parts(parts, body);
            (headers, next.run(request).await).into_response()
        }
    }

    fn clone_box(&self) -> Box<dyn Middleware> {
        Box::new(self.clone())
    }
}

impl From<Cors> for Box<dyn Middleware> {
    fn from(value: Cors) -> Box<dyn Middleware> {
        value.clone_box()
    }
}

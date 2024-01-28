use std::convert::Infallible;

use http::HeaderMap;
use mincat_core::{
    request::{FromRequestParts, Parts},
    response::{IntoResponse, IntoResponseParts, Response},
};

use super::{cookies_from_request, set_cookies};

pub struct CookieJar {
    jar: cookie::CookieJar,
}

impl CookieJar {
    fn from_headers(headers: &HeaderMap) -> Self {
        let mut jar = cookie::CookieJar::new();
        for cookie in cookies_from_request(headers) {
            jar.add_original(cookie);
        }
        Self { jar }
    }

    pub fn get(&self, name: &str) -> Option<&cookie::Cookie<'static>> {
        self.jar.get(name)
    }

    #[must_use]
    pub fn remove<C: Into<cookie::Cookie<'static>>>(mut self, cookie: C) -> Self {
        self.jar.remove(cookie);
        self
    }

    #[must_use]
    #[allow(clippy::should_implement_trait)]
    pub fn add<C: Into<cookie::Cookie<'static>>>(mut self, cookie: C) -> Self {
        self.jar.add(cookie);
        self
    }
}

#[async_trait::async_trait]
impl FromRequestParts for CookieJar {
    type Error = Infallible;

    async fn from_request_parts(parts: &mut Parts) -> Result<Self, Self::Error> {
        Ok(Self::from_headers(&parts.headers))
    }
}

impl IntoResponse for CookieJar {
    fn into_response(self) -> Response {
        let mut res = ().into_response();
        set_cookies(self.jar, res.headers_mut());
        res
    }
}

impl IntoResponseParts for CookieJar {
    fn into_response_parts(self, mut response: Response) -> Response {
        set_cookies(self.jar, response.headers_mut());
        response
    }
}

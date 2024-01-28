use cookie::{Cookie, Key, SignedJar};
use http::HeaderMap;
use mincat_core::{
    error::Error,
    request::{FromRequestParts, Parts},
    response::{IntoResponse, IntoResponseParts, Response},
};

use super::{cookies_from_request, set_cookies, CookieKey};

pub struct SignedCookieJar {
    jar: cookie::CookieJar,
    key: Key,
}

#[async_trait::async_trait]
impl FromRequestParts for SignedCookieJar {
    type Error = Error;

    async fn from_request_parts(parts: &mut Parts) -> Result<Self, Self::Error> {
        let CookieKey(key) = parts
            .extensions
            .get::<CookieKey>()
            .ok_or(Error::new("missing state Cookiekey"))?;

        Ok(SignedCookieJar::from_headers(&parts.headers, key))
    }
}

impl SignedCookieJar {
    pub fn from_headers(headers: &HeaderMap, key: &Key) -> Self {
        let mut jar = cookie::CookieJar::new();
        let mut signed_jar = jar.signed_mut(key);
        for cookie in cookies_from_request(headers) {
            if let Some(cookie) = signed_jar.verify(cookie) {
                signed_jar.add_original(cookie);
            }
        }

        Self {
            jar,
            key: key.clone(),
        }
    }

    pub fn get(&self, name: &str) -> Option<Cookie<'static>> {
        self.signed_jar().get(name)
    }

    #[must_use]
    pub fn remove<C: Into<Cookie<'static>>>(mut self, cookie: C) -> Self {
        self.signed_jar_mut().remove(cookie);
        self
    }

    #[must_use]
    #[allow(clippy::should_implement_trait)]
    pub fn add<C: Into<Cookie<'static>>>(mut self, cookie: C) -> Self {
        self.signed_jar_mut().add(cookie);
        self
    }

    fn signed_jar(&self) -> SignedJar<&'_ cookie::CookieJar> {
        self.jar.signed(&self.key)
    }

    fn signed_jar_mut(&mut self) -> SignedJar<&'_ mut cookie::CookieJar> {
        self.jar.signed_mut(&self.key)
    }
}

impl IntoResponse for SignedCookieJar {
    fn into_response(self) -> Response {
        let mut res = ().into_response();
        set_cookies(self.jar, res.headers_mut());
        res
    }
}

impl IntoResponseParts for SignedCookieJar {
    fn into_response_parts(self, mut response: Response) -> Response {
        set_cookies(self.jar, response.headers_mut());
        response
    }
}

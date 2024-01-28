use cookie::{Cookie, Key, PrivateJar};
use http::HeaderMap;
use mincat_core::{
    error::Error,
    request::{FromRequestParts, Parts},
    response::{IntoResponse, IntoResponseParts, Response},
};

use super::{cookies_from_request, set_cookies, CookieKey};

pub struct PrivateCookieJar {
    jar: cookie::CookieJar,
    key: Key,
}

#[async_trait::async_trait]
impl FromRequestParts for PrivateCookieJar {
    type Error = Error;

    async fn from_request_parts(parts: &mut Parts) -> Result<Self, Self::Error> {
        let CookieKey(key) = parts
            .extensions
            .get::<CookieKey>()
            .ok_or(Error::new("missing state Cookiekey"))?;

        Ok(PrivateCookieJar::from_headers(&parts.headers, key))
    }
}

impl PrivateCookieJar {
    fn from_headers(headers: &HeaderMap, key: &Key) -> Self {
        let mut jar = cookie::CookieJar::new();
        let mut private_jar = jar.private_mut(key);
        for cookie in cookies_from_request(headers) {
            if let Some(cookie) = private_jar.decrypt(cookie) {
                private_jar.add_original(cookie);
            }
        }

        Self {
            jar,
            key: key.clone(),
        }
    }

    pub fn get(&self, name: &str) -> Option<Cookie<'static>> {
        self.private_jar().get(name)
    }

    #[must_use]
    pub fn remove<C: Into<Cookie<'static>>>(mut self, cookie: C) -> Self {
        self.private_jar_mut().remove(cookie);
        self
    }

    #[must_use]
    #[allow(clippy::should_implement_trait)]
    pub fn add<C: Into<Cookie<'static>>>(mut self, cookie: C) -> Self {
        self.private_jar_mut().add(cookie);
        self
    }

    fn private_jar(&self) -> PrivateJar<&'_ cookie::CookieJar> {
        self.jar.private(&self.key)
    }

    fn private_jar_mut(&mut self) -> PrivateJar<&'_ mut cookie::CookieJar> {
        self.jar.private_mut(&self.key)
    }
}

impl IntoResponse for PrivateCookieJar {
    fn into_response(self) -> Response {
        let mut res = ().into_response();
        set_cookies(self.jar, res.headers_mut());
        res
    }
}

impl IntoResponseParts for PrivateCookieJar {
    fn into_response_parts(self, mut response: Response) -> Response {
        set_cookies(self.jar, response.headers_mut());
        response
    }
}

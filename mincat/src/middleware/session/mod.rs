use cookie::{Cookie, CookieJar};
use http::header::{COOKIE, SET_COOKIE};
use mincat_core::{
    error::Error, middleware::Middleware, next::Next, request::Request, response::Response,
};

use crate::extract::{Session, SessionStore};

mod db;
mod memory;
mod redis;

pub use memory::MemorySession;

pub struct StoreSession(Box<dyn SessionStore>);

impl StoreSession {
    pub fn from<T>(value: T) -> Self
    where
        T: SessionStore + 'static,
    {
        Self(Box::new(value))
    }
}

#[async_trait::async_trait]
impl Middleware for StoreSession {
    async fn call(self: Box<Self>, mut request: Request, next: Next) -> Response {
        let mut jar = CookieJar::new();

        let iter = request
            .headers()
            .get_all(COOKIE)
            .into_iter()
            .filter_map(|value| value.to_str().ok())
            .flat_map(|value| value.split(';'))
            .filter_map(|cookie| Cookie::parse_encoded(cookie.to_owned()).ok());

        for cookie in iter {
            jar.add_original(cookie);
        }

        jar.add(("name", "222"));
        let session = Session {
            store: self.0.clone_box(),
            key: "sd".to_string(),
        };

        request.extensions_mut().insert(session);

        let mut res = next.run(request).await;

        for cookie in jar.delta() {
            dbg!(cookie.encoded().to_string());
            if let Ok(header_value) = cookie.encoded().to_string().parse() {
                res.headers_mut().append(SET_COOKIE, header_value);
            }
        }

        res
    }

    fn clone_box(&self) -> Box<dyn Middleware> {
        Box::new(StoreSession(self.0.clone_box()))
    }
}

impl From<StoreSession> for Box<dyn Middleware> {
    fn from(value: StoreSession) -> Box<dyn Middleware> {
        value.clone_box()
    }
}

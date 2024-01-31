use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use http::StatusCode;
use mincat_core::{
    middleware::Middleware,
    next::Next,
    request::{FromRequestParts, Request},
    response::{IntoResponse, Response},
};
use tokio::sync::RwLock;

use crate::extract::{
    cookie::{Cookie, PrivateCookieJar},
    Session,
};

mod memory;
mod mysql;
mod postgres;
mod redis;
mod sess;

#[cfg(feature = "session-postgres")]
pub use postgres::*;

#[cfg(feature = "session-mysql")]
pub use mysql::*;

#[cfg(feature = "session-memory")]
pub use memory::*;

#[cfg(feature = "session-redis")]
pub use redis::*;

pub(crate) use sess::SessionStore;

pub struct StoreSession {
    store: Arc<RwLock<Box<dyn SessionStore>>>,
    init_tag: Arc<AtomicBool>,
}

impl StoreSession {
    pub fn from<T>(value: T) -> Self
    where
        T: SessionStore + 'static,
    {
        Self {
            store: Arc::new(RwLock::new(Box::new(value))),
            init_tag: Arc::new(AtomicBool::new(false)),
        }
    }

    async fn init(&self) -> Result<(), Response> {
        if !self.init_tag.load(Ordering::SeqCst) {
            self.store
                .write()
                .await
                .init()
                .await
                .map_err(|e| e.into_response())?;
            self.init_tag.store(true, Ordering::SeqCst);
        }
        Ok(())
    }

    async fn has_session(&self, session_id: &str) -> Result<bool, Response> {
        self.store
            .read()
            .await
            .has_session(session_id)
            .await
            .map_err(|e| e.into_response())
    }

    async fn register_key(&self, session_id: &str) -> Result<(), Response> {
        self.store
            .read()
            .await
            .register_key(session_id)
            .await
            .map_err(|e| e.into_response())
    }

    async fn new_session(
        &self,
        cookie: PrivateCookieJar,
    ) -> Result<(PrivateCookieJar, Session, String), Response> {
        let (mut session_id, is_new) = if let Some(session) = cookie.get("session") {
            (session.value().to_owned(), false)
        } else {
            (uuid::Uuid::new_v4().to_string(), true)
        };

        if is_new {
            loop {
                if self.has_session(&session_id).await? {
                    session_id = uuid::Uuid::new_v4().to_string();
                } else {
                    self.register_key(&session_id).await?;
                    let session = Session {
                        store: self.store.read().await.clone_box(),
                        session_id: session_id.clone(),
                    };
                    return Ok((cookie, session, session_id));
                }
            }
        } else if self.has_session(&session_id).await? {
            let session = Session {
                store: self.store.read().await.clone_box(),
                session_id: session_id.clone(),
            };
            Ok((cookie, session, session_id))
        } else {
            let cookie = cookie.remove("session");
            Err((StatusCode::UNAUTHORIZED, cookie, "session expired").into_response())
        }
    }
}

async fn handle(
    store_session: StoreSession,
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    store_session.init().await?;
    let (mut parts, body) = request.into_parts();
    let cookie = PrivateCookieJar::from_request_parts(&mut parts)
        .await
        .map_err(|e| e.into_response())?;
    let mut request = Request::from_parts(parts, body);

    let (cookie, session, session_id) = store_session
        .new_session(cookie)
        .await
        .map_err(|e| e.into_response())?;
    request.extensions_mut().insert(session);

    let response = next.run(request).await;

    let new_cookie = Cookie::build(("session", session_id.to_string())).http_only(true);
    let cookie = cookie.add(new_cookie);
    store_session
        .store
        .read()
        .await
        .update_exp(&session_id)
        .await
        .map_err(|e| e.into_response())?;
    Ok((cookie, response).into_response())
}

#[async_trait::async_trait]
impl Middleware for StoreSession {
    async fn call(self: Box<Self>, request: Request, next: Next) -> Response {
        handle(*self, request, next).await.into_response()
    }

    fn clone_box(&self) -> Box<dyn Middleware> {
        Box::new(StoreSession {
            store: self.store.clone(),
            init_tag: self.init_tag.clone(),
        })
    }
}

impl From<StoreSession> for Box<dyn Middleware> {
    fn from(value: StoreSession) -> Box<dyn Middleware> {
        value.clone_box()
    }
}

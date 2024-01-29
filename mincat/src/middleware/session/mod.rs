use http::StatusCode;
use mincat_core::{
    middleware::Middleware,
    next::Next,
    request::{FromRequestParts, Request},
    response::{IntoResponse, Response},
};

use crate::extract::{
    cookie::{Cookie, PrivateCookieJar},
    Session, SessionStore,
};

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
                if self
                    .0
                    .has_session(&session_id)
                    .await
                    .map_err(|e| e.into_response())?
                {
                    session_id = uuid::Uuid::new_v4().to_string();
                } else {
                    self.0
                        .register_key(&session_id)
                        .await
                        .map_err(|e| e.into_response())?;
                    let session = Session {
                        store: self.0.clone_box(),
                        session_id: session_id.clone(),
                    };
                    return Ok((cookie, session, session_id));
                }
            }
        } else if self
            .0
            .has_session(&session_id)
            .await
            .map_err(|e| e.into_response())?
        {
            let session = Session {
                store: self.0.clone_box(),
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
    store_session: &StoreSession,
    request: Request,
    next: Next,
) -> Result<Response, Response> {
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

    let new_cookie = Cookie::build(("session", session_id)).http_only(true);
    let cookie = cookie.add(new_cookie);
    Ok((cookie, response).into_response())
}

#[async_trait::async_trait]
impl Middleware for StoreSession {
    async fn call(self: Box<Self>, request: Request, next: Next) -> Response {
        handle(&self, request, next).await.into_response()
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

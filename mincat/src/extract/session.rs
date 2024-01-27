use mincat_core::{
    error::Error,
    request::{FromRequestParts, Parts},
    response::IntoResponse,
};
use serde::Serialize;

use super::ExtractError;

#[async_trait::async_trait]
pub trait SessionStore: Send + Sync {
    async fn exists(&self, key: &str) -> Result<(), Error>;

    async fn delete_by_key(&self, key: &str) -> Result<(), Error>;

    async fn delete_expiry(&self) -> Result<(), Error>;

    async fn delete_all(&self) -> Result<(), Error>;

    fn clone_box(&self) -> Box<dyn SessionStore>;
}

pub struct Session {
    pub(crate) store: Box<dyn SessionStore>,
    pub(crate) key: String,
}

impl Clone for Session {
    fn clone(&self) -> Self {
        Self {
            store: self.store.clone_box(),
            key: self.key.clone(),
        }
    }
}

impl Session {
    pub async fn set(&mut self, key: &str, value: impl Serialize) -> Result<(), ExtractError> {
        todo!()
    }

    pub async fn get(&mut self, key: &str, value: impl Serialize) -> Result<(), ExtractError> {
        todo!()
    }
}

#[async_trait::async_trait]
impl FromRequestParts for Session {
    type Error = ExtractError;

    async fn from_request_parts(parts: &mut Parts) -> Result<Self, Self::Error> {
        parts
            .extensions
            .get::<Session>()
            .cloned()
            .ok_or(ExtractError("get session failed".to_string()))
    }
}

use mincat_core::{
    error::Error,
    request::{FromRequestParts, Parts},
};
use serde::{de::DeserializeOwned, Serialize};

#[async_trait::async_trait]
pub trait SessionStore: Send + Sync {
    async fn has_session(&self, session_id: &str) -> Result<bool, Error>;

    async fn register_key(&self, session_id: &str) -> Result<(), Error>;

    async fn set(&mut self, session_id: &str, key: &str, value: &str) -> Result<(), Error>;

    async fn get(&mut self, session_id: &str, key: &str) -> Result<Option<String>, Error>;

    fn clone_box(&self) -> Box<dyn SessionStore>;
}

pub struct Session {
    pub(crate) store: Box<dyn SessionStore>,
    pub(crate) session_id: String,
}

impl Clone for Session {
    fn clone(&self) -> Self {
        Self {
            store: self.store.clone_box(),
            session_id: self.session_id.clone(),
        }
    }
}

impl Session {
    pub async fn set<T: Serialize>(&mut self, key: &str, value: T) -> Result<(), Error> {
        let value = serde_json::to_string(&value).map_err(Error::new)?;
        self.store.set(&self.session_id, key, &value).await?;
        Ok(())
    }

    pub async fn get<T: DeserializeOwned>(&mut self, key: &str) -> Result<Option<T>, Error> {
        let value = self.store.get(&self.session_id, key).await?;
        match value {
            Some(value) => Ok(Some(
                serde_json::from_str::<T>(value.as_str()).map_err(Error::new)?,
            )),
            None => Ok(None),
        }
    }
}

#[async_trait::async_trait]
impl FromRequestParts for Session {
    type Error = Error;

    async fn from_request_parts(parts: &mut Parts) -> Result<Self, Self::Error> {
        parts
            .extensions
            .get::<Session>()
            .cloned()
            .ok_or(Error::new("get session failed"))
    }
}

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use mincat_core::error::Error;

use crate::extract::SessionStore;

#[derive(Clone, Default, Debug)]
pub struct MemorySession {
    store: Arc<RwLock<HashMap<String, HashMap<String, String>>>>,
}

#[async_trait::async_trait]
impl SessionStore for MemorySession {
    async fn has_session(&self, session_id: &str) -> Result<bool, Error> {
        let store = self.store.read().map_err(|e| Error::new(e.to_string()))?;
        Ok(store.contains_key(session_id))
    }

    async fn register_key(&self, session_id: &str) -> Result<(), Error> {
        let mut store = self.store.write().map_err(|e| Error::new(e.to_string()))?;
        store.insert(session_id.to_string(), HashMap::new());
        Ok(())
    }

    async fn set(&mut self, session_id: &str, key: &str, value: &str) -> Result<(), Error> {
        let mut store = self.store.write().map_err(|e| Error::new(e.to_string()))?;
        let hm = store
            .entry(session_id.to_string())
            .or_insert(HashMap::new());
        hm.insert(key.to_string(), value.to_string());
        Ok(())
    }

    async fn get(&mut self, session_id: &str, key: &str) -> Result<Option<String>, Error> {
        let store = self.store.read().map_err(|e| Error::new(e.to_string()))?;
        match store.get(session_id) {
            Some(hm) => Ok(hm.get(key).map(|e| e.to_owned())),
            None => Ok(None),
        }
    }

    fn clone_box(&self) -> Box<dyn SessionStore> {
        Box::new(self.clone())
    }
}

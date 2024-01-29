use std::{
    collections::HashMap,
    fmt::Debug,
    hash::Hash,
    sync::{Arc, RwLock},
};

use chrono::{DateTime, Duration, Utc};
use derive_builder::Builder;
use mincat_core::error::Error;

use super::sess::SessionStore;

#[derive(Debug, Eq, Clone)]
struct SessionKey {
    key: String,
    exp: DateTime<Utc>,
}

impl SessionKey {
    fn new(key: &str, age: i64) -> Self {
        let dur = Duration::seconds(age);
        Self {
            key: key.to_string(),
            exp: Utc::now() + dur,
        }
    }
}

impl Hash for SessionKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key.hash(state);
    }
}

impl PartialEq for SessionKey {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

#[derive(Clone, Debug, Builder)]
pub struct MemorySession {
    #[builder(setter(skip))]
    store: Arc<RwLock<HashMap<SessionKey, HashMap<String, String>>>>,
    #[builder(default = "3600")]
    age: i64,
    #[builder(default = "60")]
    interval: u64,
}

impl MemorySession {
    fn session_key(&self, session_id: &str) -> SessionKey {
        SessionKey::new(session_id, self.age)
    }
}

#[async_trait::async_trait]
impl SessionStore for MemorySession {
    async fn has_session(&self, session_id: &str) -> Result<bool, Error> {
        let store = self.store.read().map_err(|e| Error::new(e.to_string()))?;
        let kv = store.get_key_value(&self.session_key(session_id));
        match kv {
            Some((k, _)) => {
                if Utc::now() > k.exp {
                    Ok(false)
                } else {
                    Ok(true)
                }
            }
            None => Ok(false),
        }
    }

    async fn register_key(&self, session_id: &str) -> Result<(), Error> {
        let mut store = self.store.write().map_err(|e| Error::new(e.to_string()))?;
        store.insert(self.session_key(session_id), HashMap::new());
        Ok(())
    }

    async fn set(&self, session_id: &str, key: &str, value: &str) -> Result<(), Error> {
        let mut store = self.store.write().map_err(|e| Error::new(e.to_string()))?;
        let hm = store
            .entry(self.session_key(session_id))
            .or_insert(HashMap::new());
        hm.insert(key.to_string(), value.to_string());
        Ok(())
    }

    async fn get(&self, session_id: &str, key: &str) -> Result<Option<String>, Error> {
        let store = self.store.read().map_err(|e| Error::new(e.to_string()))?;
        match store.get(&self.session_key(session_id)) {
            Some(hm) => Ok(hm.get(key).map(|e| e.to_owned())),
            None => Ok(None),
        }
    }

    async fn delete_exp(&self) -> Result<(), Error> {
        let mut store = self.store.write().map_err(|e| Error::new(e.to_string()))?;
        let now = Utc::now();
        let mut exp_session_keys = vec![];
        for session_key in store.keys() {
            if now > session_key.exp {
                exp_session_keys.push(session_key.clone());
            }
        }

        for session_key in exp_session_keys {
            store.remove(&session_key);
        }

        Ok(())
    }

    async fn update_exp(&self, session_id: &str) -> Result<(), Error> {
        let mut store = self.store.write().map_err(|e| Error::new(e.to_string()))?;
        let value = store.get(&self.session_key(session_id)).cloned();

        if let Some(value) = value {
            let session_key = self.session_key(session_id);
            store.remove(&session_key);
            store.insert(session_key, value.clone());
        }

        Ok(())
    }

    fn get_delete_exp_task_boot_tag(&self) -> bool {
        true
    }

    fn get_delete_exp_task_interval(&self) -> u64 {
        self.interval
    }

    fn clone_box(&self) -> Box<dyn SessionStore> {
        Box::new(self.clone())
    }
}

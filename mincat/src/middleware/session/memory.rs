use mincat_core::error::Error;

use crate::extract::SessionStore;

pub struct MemorySession;

#[async_trait::async_trait]
impl SessionStore for MemorySession {
    async fn exists(&self, key: &str) -> Result<(), Error> {
        todo!()
    }

    async fn delete_by_key(&self, key: &str) -> Result<(), Error> {
        todo!()
    }

    async fn delete_expiry(&self) -> Result<(), Error> {
        todo!()
    }

    async fn delete_all(&self) -> Result<(), Error> {
        todo!()
    }

    fn clone_box(&self) -> Box<dyn SessionStore> {
        Box::new(MemorySession)
    }
}

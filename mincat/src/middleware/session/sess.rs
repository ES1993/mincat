use mincat_core::error::Error;

#[async_trait::async_trait]
pub trait SessionStore: Send + Sync + 'static {
    async fn has_session(&self, session_id: &str) -> Result<bool, Error>;

    async fn register_key(&self, session_id: &str) -> Result<(), Error>;

    async fn set(&self, session_id: &str, key: &str, value: &str) -> Result<(), Error>;

    async fn get(&self, session_id: &str, key: &str) -> Result<Option<String>, Error>;

    async fn delete_exp(&self) -> Result<(), Error>;

    async fn update_exp(&self, session_id: &str) -> Result<(), Error>;

    fn get_delete_exp_task_boot_tag(&self) -> bool;

    fn get_delete_exp_task_interval(&self) -> u64;

    fn init(&self) {
        if self.get_delete_exp_task_boot_tag() {
            let self_task = self.clone_box();
            let interval = self.get_delete_exp_task_interval();
            tokio::spawn(async move {
                let mut interval =
                    tokio::time::interval(tokio::time::Duration::from_secs(interval));
                loop {
                    if let Err(e) = self_task.delete_exp().await {
                        tracing::error!("session delete exp key task error:{}", e);
                    }

                    interval.tick().await;
                }
            });
        }
    }

    fn clone_box(&self) -> Box<dyn SessionStore>;
}

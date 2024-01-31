use derive_builder::Builder;
use mincat_core::error::Error;
use redis::{aio::ConnectionLike, cluster::ClusterClient, Client};
use redis_pool::{ClusterRedisPool, RedisPool, SingleRedisPool};

use super::SessionStore;

#[derive(Clone, Builder)]
pub struct RedisSession {
    #[builder(setter(skip))]
    pool: Option<SingleRedisPool>,
    url: String,
    #[builder(default = "3600")]
    age: i64,
    #[builder(default = "String::from(\"mincat:session\")")]
    prefix: String,
}

impl RedisSession {
    fn session_key(&self, session_id: &str) -> String {
        format!("{}:{}", self.prefix, session_id)
    }

    async fn get_conn(&self) -> Result<impl ConnectionLike, Error> {
        self.pool
            .clone()
            .unwrap()
            .aquire()
            .await
            .map_err(Error::new)
    }
}

async fn has_session(session_key: &str, mut conn: impl ConnectionLike) -> Result<bool, Error> {
    redis::cmd("EXISTS")
        .arg(session_key)
        .query_async(&mut conn)
        .await
        .map_err(Error::new)
}

async fn register_key(
    session_key: &str,
    age: i64,
    mut conn: impl ConnectionLike,
) -> Result<(), Error> {
    redis::pipe()
        .hset(session_key, "", "")
        .expire(session_key, age)
        .query_async(&mut conn)
        .await
        .map_err(Error::new)?;
    Ok(())
}

async fn set(
    session_key: &str,
    key: &str,
    value: &str,
    mut conn: impl ConnectionLike,
) -> Result<(), Error> {
    redis::pipe()
        .hset(session_key, key, value)
        .query_async(&mut conn)
        .await
        .map_err(Error::new)?;
    Ok(())
}

async fn get(
    session_key: &str,
    key: &str,
    mut conn: impl ConnectionLike,
) -> Result<Option<String>, Error> {
    redis::Cmd::hget(session_key, key)
        .query_async(&mut conn)
        .await
        .map_err(Error::new)
}

async fn update_exp(session_key: &str, age: i64, conn: impl ConnectionLike) -> Result<(), Error> {
    register_key(session_key, age, conn).await
}

#[async_trait::async_trait]
impl SessionStore for RedisSession {
    async fn init(&mut self) -> Result<(), Error> {
        let client = Client::open(self.url.as_str()).expect("can't connect to redis");
        self.pool = Some(RedisPool::from(client));
        Ok(())
    }

    async fn has_session(&self, session_id: &str) -> Result<bool, Error> {
        has_session(&self.session_key(session_id), self.get_conn().await?).await
    }

    async fn register_key(&self, session_id: &str) -> Result<(), Error> {
        register_key(
            &self.session_key(session_id),
            self.age,
            self.get_conn().await?,
        )
        .await
    }

    async fn set(&self, session_id: &str, key: &str, value: &str) -> Result<(), Error> {
        set(
            &self.session_key(session_id),
            key,
            value,
            self.get_conn().await?,
        )
        .await
    }

    async fn get(&self, session_id: &str, key: &str) -> Result<Option<String>, Error> {
        get(&self.session_key(session_id), key, self.get_conn().await?).await
    }

    async fn delete_exp(&self) -> Result<(), Error> {
        Ok(())
    }

    async fn update_exp(&self, session_id: &str) -> Result<(), Error> {
        update_exp(
            &self.session_key(session_id),
            self.age,
            self.get_conn().await?,
        )
        .await
    }

    fn get_delete_exp_task_boot_tag(&self) -> bool {
        false
    }

    fn get_delete_exp_task_interval(&self) -> u64 {
        0
    }

    fn clone_box(&self) -> Box<dyn SessionStore> {
        Box::new(self.clone())
    }
}

#[derive(Clone, Builder)]
pub struct RedisClusterSession {
    #[builder(setter(skip))]
    pool: Option<ClusterRedisPool>,
    urls: Vec<String>,
    #[builder(default = "3600")]
    age: i64,
    #[builder(default = "String::from(\"mincat:session\")")]
    prefix: String,
}

impl RedisClusterSession {
    fn session_key(&self, session_id: &str) -> String {
        format!("{}:{}", self.prefix, session_id)
    }

    async fn get_conn(&self) -> Result<impl ConnectionLike, Error> {
        self.pool
            .clone()
            .unwrap()
            .aquire()
            .await
            .map_err(Error::new)
    }
}

#[async_trait::async_trait]
impl SessionStore for RedisClusterSession {
    async fn init(&mut self) -> Result<(), Error> {
        let urls = self.urls.iter().map(|s| s.as_ref()).collect::<Vec<_>>();
        let client = ClusterClient::new(urls).expect("can't connect to redis");
        self.pool = Some(ClusterRedisPool::from(client));
        Ok(())
    }

    async fn has_session(&self, session_id: &str) -> Result<bool, Error> {
        has_session(&self.session_key(session_id), self.get_conn().await?).await
    }

    async fn register_key(&self, session_id: &str) -> Result<(), Error> {
        register_key(
            &self.session_key(session_id),
            self.age,
            self.get_conn().await?,
        )
        .await
    }

    async fn set(&self, session_id: &str, key: &str, value: &str) -> Result<(), Error> {
        set(
            &self.session_key(session_id),
            key,
            value,
            self.get_conn().await?,
        )
        .await
    }

    async fn get(&self, session_id: &str, key: &str) -> Result<Option<String>, Error> {
        get(&self.session_key(session_id), key, self.get_conn().await?).await
    }

    async fn delete_exp(&self) -> Result<(), Error> {
        Ok(())
    }

    async fn update_exp(&self, session_id: &str) -> Result<(), Error> {
        update_exp(
            &self.session_key(session_id),
            self.age,
            self.get_conn().await?,
        )
        .await
    }

    fn get_delete_exp_task_boot_tag(&self) -> bool {
        false
    }

    fn get_delete_exp_task_interval(&self) -> u64 {
        0
    }

    fn clone_box(&self) -> Box<dyn SessionStore> {
        Box::new(self.clone())
    }
}

use chrono::{Duration, Utc};
use derive_builder::Builder;
use mincat_core::error::Error;
use sqlx::{pool::Pool, PgPool, Postgres};

use super::SessionStore;

#[derive(Clone, Debug, Builder)]
pub struct PostgresSession {
    url: String,
    #[builder(default = "String::from(\"mincat_session\")")]
    table_name: String,
    #[builder(default = "3600")]
    age: i64,
    #[builder(default = "60")]
    interval: u64,
    #[builder(setter(skip))]
    pool: Option<Pool<Postgres>>,
}

impl PostgresSession {
    fn get_conn(&self) -> Pool<Postgres> {
        let pool = self.pool.clone();
        pool.unwrap()
    }

    fn get_exp(&self) -> i64 {
        let dur = Duration::seconds(self.age);
        (Utc::now() + dur).timestamp()
    }
}

#[async_trait::async_trait]
impl SessionStore for PostgresSession {
    async fn init(&mut self) -> Result<(), Error> {
        self.pool = Some(PgPool::connect(&self.url).await.map_err(Error::new)?);

        sqlx::query(
            &r#"
            CREATE TABLE IF NOT EXISTS %%TABLE_NAME%% (
                session TEXT NOT NULL,
                key TEXT NOT NULL,
                value TEXT NOT NULL,
                exp INTEGER NULL,
                PRIMARY KEY (session, key)
            )
            "#
            .replace("%%TABLE_NAME%%", &self.table_name),
        )
        .execute(&self.get_conn())
        .await
        .map_err(Error::new)?;

        self.delete_exp_task();

        Ok(())
    }

    async fn has_session(&self, session_id: &str) -> Result<bool, Error> {
        let result: Option<(i64,)> = sqlx::query_as(
            &r#"
            SELECT COUNT(*) 
            FROM %%TABLE_NAME%%
            WHERE session = $1 
            AND exp > $2
            "#
            .replace("%%TABLE_NAME%%", &self.table_name),
        )
        .bind(session_id)
        .bind(Utc::now().timestamp())
        .fetch_optional(&self.get_conn())
        .await
        .map_err(Error::new)?;

        if let Some((count,)) = result {
            if count > 0 {
                return Ok(true);
            }
        }

        Ok(false)
    }

    async fn register_key(&self, session_id: &str) -> Result<(), Error> {
        sqlx::query(
            &r#"
            INSERT INTO %%TABLE_NAME%% (session, key, value, exp) 
            VALUES ($1, $2, $3, $4)
            "#
            .replace("%%TABLE_NAME%%", &self.table_name),
        )
        .bind(session_id)
        .bind("")
        .bind("")
        .bind(self.get_exp())
        .execute(&self.get_conn())
        .await
        .map_err(Error::new)?;

        Ok(())
    }

    async fn set(&self, session_id: &str, key: &str, value: &str) -> Result<(), Error> {
        sqlx::query(
            &r#"
            INSERT INTO %%TABLE_NAME%% (session, key, value) 
            SELECT $1, $2, $3
            ON CONFLICT(session, key) DO UPDATE SET
            value = EXCLUDED.value
            "#
            .replace("%%TABLE_NAME%%", &self.table_name),
        )
        .bind(session_id)
        .bind(key)
        .bind(value)
        .execute(&self.get_conn())
        .await
        .map_err(Error::new)?;

        Ok(())
    }

    async fn get(&self, session_id: &str, key: &str) -> Result<Option<String>, Error> {
        let res: Option<(String,)> = sqlx::query_as(
            &r#"
            SELECT value 
            FROM %%TABLE_NAME%%
            WHERE session = $1
            AND key = $2
            "#
            .replace("%%TABLE_NAME%%", &self.table_name),
        )
        .bind(session_id)
        .bind(key)
        .fetch_optional(&self.get_conn())
        .await
        .map_err(Error::new)?;

        if let Some((text,)) = res {
            return Ok(Some(text));
        }
        Ok(None)
    }

    async fn delete_exp(&self) -> Result<(), Error> {
        sqlx::query(
            &r#"
            DELETE 
            FROM %%TABLE_NAME%% 
            WHERE session IN (
                SELECT session 
                FROM %%TABLE_NAME%%
                WHERE exp < $1
            )
            "#
            .replace("%%TABLE_NAME%%", &self.table_name),
        )
        .bind(Utc::now().timestamp())
        .execute(&self.get_conn())
        .await
        .map_err(Error::new)?;

        Ok(())
    }

    async fn update_exp(&self, session_id: &str) -> Result<(), Error> {
        sqlx::query(
            &r#"
            INSERT INTO %%TABLE_NAME%% (session, key, value, exp) 
            SELECT $1, $2, $3, $4
            ON CONFLICT(session, key) DO UPDATE SET
            value = EXCLUDED.value,
            exp = EXCLUDED.exp
            "#
            .replace("%%TABLE_NAME%%", &self.table_name),
        )
        .bind(session_id)
        .bind("")
        .bind("")
        .bind(self.get_exp())
        .execute(&self.get_conn())
        .await
        .map_err(Error::new)?;

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

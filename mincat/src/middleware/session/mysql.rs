use chrono::{Duration, Utc};
use derive_builder::Builder;
use mincat_core::error::Error;
use sqlx::{pool::Pool, MySql, MySqlPool};

use super::SessionStore;

#[derive(Clone, Debug, Builder)]
pub struct MysqlSession {
    #[builder(setter(into))]
    url: String,
    #[builder(setter(into), default = "String::from(\"mincat_session\")")]
    table_name: String,
    #[builder(default = "3600")]
    age: i64,
    #[builder(default = "60")]
    interval: u64,
    #[builder(setter(skip))]
    pool: Option<Pool<MySql>>,
}

impl MysqlSession {
    fn get_conn(&self) -> Pool<MySql> {
        let pool = self.pool.clone();
        pool.unwrap()
    }

    fn get_exp(&self) -> i64 {
        let dur = Duration::seconds(self.age);
        (Utc::now() + dur).timestamp()
    }
}

#[async_trait::async_trait]
impl SessionStore for MysqlSession {
    async fn init(&mut self) -> Result<(), Error> {
        self.pool = Some(MySqlPool::connect(&self.url).await.map_err(Error::new)?);

        sqlx::query(
            &r#"
            CREATE TABLE IF NOT EXISTS %%TABLE_NAME%% (
                session TEXT NOT NULL,
                `key` TEXT NOT NULL,
                value TEXT NOT NULL,
                exp INTEGER NULL,
                PRIMARY KEY (session(36), `key`(36))
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
            WHERE session = ?
            AND exp > ?
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
            INSERT INTO %%TABLE_NAME%% (session, `key`, value, exp) 
            VALUES (?, ?, ?, ?)
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
            INSERT INTO %%TABLE_NAME%% (session, `key`, value) 
            SELECT ?, ?, ?
            ON DUPLICATE KEY UPDATE
            value = VALUES(value)
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
            WHERE session = ?
            AND `key` = ?
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
        let result: Vec<(String,)> = sqlx::query_as(
            &r#"
            SELECT session 
            FROM %%TABLE_NAME%%
            WHERE exp < ?
            "#
            .replace("%%TABLE_NAME%%", &self.table_name),
        )
        .bind(Utc::now().timestamp())
        .fetch_all(&self.get_conn())
        .await
        .map_err(Error::new)?;

        let result: Vec<String> = result.into_iter().map(|(s,)| s).collect();

        if !result.is_empty() {
            let sql = format!(
                r#"
                DELETE 
                FROM %%TABLE_NAME%% 
                WHERE session IN ({})
                "#,
                result
                    .iter()
                    .map(|s| format!("\"{}\"", s))
                    .collect::<Vec<_>>()
                    .join(",")
            );
            let sql = sql.replace("%%TABLE_NAME%%", &self.table_name);
            sqlx::query(&sql)
                .execute(&self.get_conn())
                .await
                .map_err(Error::new)?;
        }

        Ok(())
    }

    async fn update_exp(&self, session_id: &str) -> Result<(), Error> {
        sqlx::query(
            &r#"
            INSERT INTO %%TABLE_NAME%% (session, `key`, value, exp) 
            SELECT ?, ?, ?, ?
            ON DUPLICATE KEY UPDATE
            value = VALUES(value),
            exp = VALUES(exp)
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

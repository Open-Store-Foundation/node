use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Error, Sqlite, SqlitePool, Transaction};
use std::time::Duration;

pub type SqlxError = Error;
pub type SqlxResult<T> = Result<T, Error>;

pub struct SqliteClient {
    pool: SqlitePool
}

impl SqliteClient {

    pub async fn create(path: String) -> SqlxResult<SqliteClient> {
        let pool = SqlitePoolOptions::new()
            .max_connections(3)
            .idle_timeout(Duration::from_secs(10 * 50))
            .max_lifetime(Duration::from_secs(30 * 60))
            .acquire_timeout(Duration::from_secs(5))
            .test_before_acquire(false)
            .connect(path.as_str())
            .await?;

        return Ok(SqliteClient { pool });
    }

    pub async fn ping(&self) -> SqlxResult<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn start(
        &self
    ) -> SqlxResult<Transaction<'static, Sqlite>> {
        let transaction = self.pool().begin()
            .await?;

        return Ok(transaction);
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

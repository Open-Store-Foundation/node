use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;
use sqlx::{Error, Postgres, Transaction};

pub type SqlxError = Error;
pub type SqlxResult<T> = Result<T, Error>;

#[derive(Clone)]
pub struct PgClient {
    pool: PgPool,
}

impl PgClient {

    pub async fn connect(database_url: &str) -> SqlxResult<PgClient> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .idle_timeout(Duration::from_secs(10 * 50))
            .max_lifetime(Duration::from_secs(30 * 60))
            .acquire_timeout(Duration::from_secs(5))
            .test_before_acquire(false)
            .connect(database_url)
            .await?;

        // let options = PgConnectOptions::new()
        //     .log_statements(log::LevelFilter::Info);
        // pool.set_connect_options(options);

        Ok(PgClient { pool })
    }

    pub async fn ping(&self) -> SqlxResult<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn start(
        &self
    ) -> SqlxResult<Transaction<'static, Postgres>> {
        let transaction = self.pool().begin()
            .await?;

        return Ok(transaction);
    }

    // pub async fn migrate(&self, path: &str) -> SqlxResult<()> {
    //     println!("Running database migrations...");
    //
    //     sqlx::migrate!()
    //         .run(&self.pool)
    //         .await?;
    //
    //     println!("Migrations completed successfully.");
    //     Ok(())
    // }

    // Method to get access to the pool for repositories
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}
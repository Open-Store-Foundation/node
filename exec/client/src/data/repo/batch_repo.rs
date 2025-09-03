use crate::result::ClientResult;
use db_psql::client::PgClient;
use sqlx::{Database, Decode, PgPool, Type};

#[derive(Type, Clone)]
#[repr(i32)]
pub enum TransactionStatus {
    Confirmed = 0,
    Failure = 1,
}

pub struct TransactionBatch {
    pub from_block_number: i64,
    pub to_block_number: i64,
    pub status: TransactionStatus,
}

pub struct BatchRepo {
    client: PgClient
}

impl BatchRepo {
    pub fn new(client: PgClient) -> Self {
        return Self {
            client
        }
    }

    pub fn pool(&self) -> &PgPool {
        self.client.pool()
    }

    pub async fn save_batch(&self, batch: TransactionBatch) -> ClientResult<()> {
        let result = sqlx::query_as!(
            TransactionBatch,
            r#"
            INSERT INTO transactions_batch (
                from_block_number, to_block_number, status
            )
            
            VALUES ($1, $2, $3)
            "#,
            batch.from_block_number,
            batch.to_block_number,
            batch.status as i32
        )
            .execute(self.pool())
            .await?;

        Ok(())
    }
    
    pub async fn get_last_batch(&self) -> ClientResult<Option<TransactionBatch>> {
        let result = sqlx::query_as!(
            TransactionBatch,
            r#"
            SELECT 
                b.from_block_number,
                b.to_block_number,
                b.status as "status: _"
            FROM transactions_batch b 
            ORDER BY b.to_block_number DESC 
            LIMIT 1
            "#
        )
            .fetch_optional(self.pool())
            .await?;
        
        Ok(result)
    }
}
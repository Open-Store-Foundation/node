use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use prost::Message;
use sqlx::SqlitePool;
use tokio::sync::RwLock;
use tracing::error;
use codegen_block::block::ValidationResult;
use core_log::init_tracer;
use core_std::arc;
use db_sqlite::client::SqliteClient;
use service_sc::store::BlockState;
use crate::result::ValidatorResult;

pub struct ValidationRepo {
    client: Arc<SqliteClient>
}

impl ValidationRepo {

    pub fn new(db: &Arc<SqliteClient>) -> ValidationRepo {
        return Self {
            client: db.clone()
        };
    }

    pub fn pool(&self) -> &SqlitePool {
        self.client.pool()
    }

    pub async fn clear_all(&self) -> ValidatorResult<()> {
        sqlx::query!(
            r#"
                DELETE FROM val_block;
                DELETE FROM val_req;
            "#
        )
            .execute(self.pool())
            .await?;

        return Ok(());
    }

    pub async fn has_request(&self, request_id: u64) -> bool {
        let req = request_id as i64;
        let result= sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM val_req WHERE request_id = $1 LIMIT 1;"#,
            req
        )
            .fetch_one(self.pool())
            .await;

        let Ok(result) = result else {
            return false
        };

        return result > 0;
    }

    pub async fn is_voted(&self, block_id: u64) -> ValidatorResult<bool> {
        let block_state = self.block_state(block_id).await?;

        let Some(state) = block_state else {
            return Ok(false)
        };

        return Ok(state.at_least_voted())
    }

    pub async fn is_submitted(&self, block_id: u64) -> ValidatorResult<bool> {
        let block_state = self.block_state(block_id).await?;

        let Some(state) = block_state else {
            return Ok(false)
        };

        return Ok(state.at_least_proposed())
    }

    pub async fn next_validate_request_id(&self) -> ValidatorResult<u64> {
        let result = self.get_last_validated_request()
            .await?;

        return Ok(result + 1);
    }

    pub async fn get_last_validated_request(&self) -> ValidatorResult<u64> {
        let result: Option<i64> = sqlx::query_scalar!(
            "SELECT request_id FROM val_req ORDER BY request_id DESC LIMIT 1;",
        )
            .fetch_optional(self.pool())
            .await?;
        
        let id = result.unwrap_or(0) as u64;
        return Ok(id);
    }

    pub async fn save_request(&self, request_id: u64, data: &ValidationResult) {
        let req = request_id as i64;
        let data = data.encode_to_vec();
        
        let result = sqlx::query!(
            "INSERT INTO val_req (request_id, proto) VALUES ($1, $2)",
            req, data
        )
            .execute(self.pool())
            .await;

        if let Err(err) = result {
            error!("Failed to save request: {}", err);
        }
    }

    pub async fn get_result(&self, request_id: u64) -> Option<ValidationResult> {
        let req_id = request_id as i64;
        let result = sqlx::query!(
            "SELECT request_id, proto FROM val_req WHERE request_id == $1 LIMIT 1",
            req_id
        )
            .fetch_optional(self.pool())
            .await;
        
        let result = match result {
            Ok(result) => result,
            Err(err) => return None
        };
        
        match result {
            None => None,
            Some(row) => match ValidationResult::decode(row.proto.as_slice()) {
                Ok(result) => Some(result),
                Err(err) => None
            }
        }
    }
    
    pub async fn get_results(&self, from_req_id: u64, to_req_id: u64) -> ValidatorResult<Vec<ValidationResult>> {
        let from_req = from_req_id as i64;
        let to_req = to_req_id as i64;
        let records = sqlx::query!(
            "SELECT request_id, proto FROM val_req WHERE request_id >= ($1) AND request_id < ($2) ORDER BY request_id ASC",
            from_req, to_req
        )
            .fetch_all(self.pool())
            .await?;
        
        let mut results = Vec::with_capacity(records.len());
        
        for record in records {
            let result = ValidationResult::decode(record.proto.as_slice())?;
            results.push(result);
        }
        
        return Ok(results);
    }

    pub async fn block_state(&self, block_id: u64) -> ValidatorResult<Option<BlockState>> {
        let block = block_id as i64;
        let result: Option<i64> = sqlx::query_scalar!(
            "SELECT state FROM val_block WHERE block_id == ($1) LIMIT 1",
            block
        )
            .fetch_optional(self.pool())
            .await?;

        let Some(num) = result else {
            return Ok(None)
        };

        return Ok(Some(BlockState::from_u8(num as u8)));
    }

    pub async fn save_block(&self, block_id: u64, state: BlockState, data: &Vec<u8>) -> ValidatorResult<()> {
        let block = block_id as i64;
        let state_id = state as u8;

        let _ = sqlx::query!(
            "INSERT INTO val_block (block_id, state, proto) VALUES ($1, $2, $3)",
            block, state_id, data
        )
            .execute(self.pool())
            .await;

        return Ok(());
    }

    pub async fn update_block_state(&self, block_id: u64, state: BlockState) -> ValidatorResult<()> {
        let block = block_id as i64;
        let state_id = state as u8;
        sqlx::query!(
            "UPDATE val_block SET state = $1 WHERE block_id = $2;",
            state_id, block
        )
            .execute(self.pool())
            .await?;

        return Ok(());
    }
}


#[tokio::test]
async fn test_db() {
    // init_tracer();
    //
    // let db = arc!(SqliteClient::default());
    // db.init().expect("");
    //
    // let repo = ValidationRepo::new(&db);
    // let has_request = repo.has_request(12)
    //     .expect("");
    // let request = repo.get_last_validated_request()
    //     .expect("");
    // let state = repo.block_state(12)
    //     .expect("");
    // let requests = repo.get_requests(0, 1)
    //     .expect("");
    //
    // repo.update_block_state(12, BlockState::Discussing)
    //     .expect("");
    //
    // assert_eq!(has_request, false);
    // assert_eq!(request, 0);
    // assert_eq!(requests.len(), 0);
    // assert_eq!(state, None);
}
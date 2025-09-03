use std::cmp::min;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::error;
use client_tg::tg_alert;
use codegen_block::block::{ValidationBlock, ValidationResult};
use service_sc::obj::ScObjService;
use service_sc::store::ScStoreService;
use crate::android::validator::AndroidValidator;
use crate::data::block_repo::BlockRepo;
use crate::data::validation_repo::ValidationRepo;
use crate::result::ValidatorResult;

pub struct ValidationCase {
    validation_repo: Arc<ValidationRepo>,
    store_service: Arc<ScStoreService>,
    block_repo: Arc<BlockRepo>,
    validator: Arc<AndroidValidator>,
}

impl ValidationCase {

    pub fn new(
        validation_repo: Arc<ValidationRepo>, 
        store_service: Arc<ScStoreService>, 
        block_repo: Arc<BlockRepo>, 
        validator: Arc<AndroidValidator>
    ) -> Self {
        Self { validation_repo, store_service, block_repo, validator }
    }
    
    pub async fn validation_block(&self, block_id: u64, from: u64, to: Option<u64>) -> ValidatorResult<Option<ValidationBlock>> {
        let to = match to {
            Some(to) => to,
            None => {
                match self.next_validate_request_id(from).await {
                    Ok(to) => match to {
                        Some(value) => value,
                        None => return Ok(None),
                    },
                    Err(e) => {
                        error!("[VALIDATE_HANDLER] Failed to get next validated request: {}.", e);
                        tg_alert!(format!("[VALIDATE_HANDLER] Failed to get next validated request: {}.", e));
                        return Ok(None);
                    }
                }
            }
        };

        let mut results = self.validation_repo.get_results(from, to)
            .await?
            .into_iter()
            .map(|item| (item.request_id.clone(), item))
            .collect::<HashMap<u64, ValidationResult>>();

        let required_count = (to - from) as usize;
        let mut data = Vec::<ValidationResult>::with_capacity(required_count);

        for req_id in from..to {
            let id = (req_id - from) as usize;

            if let Some(result) = results.remove(&req_id) {
                data.insert(id, result);
                continue;
            }

            let result = self.validate_request(req_id).await;
            data.insert(id, result);
            continue;
        }

        let block = self.block_repo.create_block(block_id, data);

        return Ok(Some(block));
    }

    pub async fn validate_request(&self, request_id: u64) -> ValidationResult {
        let Ok(info) = self.store_service.get_request(request_id).await else {
            return ValidationResult::unavailable(request_id)
        };

        let result = self.validator
            .validate_request(info.req_type, info.target, request_id, info.data.0.as_ref())
            .await;

        return result;
    }

    pub async fn next_validate_request_id(&self, from: u64) -> ValidatorResult<Option<u64>> {
        let mut to = self.validation_repo.next_validate_request_id()
            .await?;

        if from >= to { // we don't have any in local storage
            let next_req_id = self.store_service.next_request_id()
                .await?;

            if from >= next_req_id {
                return Ok(None);
            } else {
                to = from + 1; // TODO v2 min(from + 128, next_req_id)
            }
        } else {
            to = min(from + 1, to) // TODO v2 min(from + 128, to)
        }

        Ok(Some(to))
    }
}

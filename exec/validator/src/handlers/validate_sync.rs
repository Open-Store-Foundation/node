use crate::android::validator::AndroidValidator;
use crate::data::block_repo::BlockRepo;
use crate::data::validation_repo::ValidationRepo;
use crate::handlers::common::validation::ValidationCase;
use crate::launcher::{ValidationContext, ValidatorEvent};
use crate::result::ValidatorResult;
use crate::utils::hasher::HasherSha256;
use crate::utils::merkle::MerkleTree;
use alloy::sol_types::SolType;
use codegen_block::block::{ValidationBlock, ValidationResult};
use core_std::trier::SyncTrier;
use prost::Message;
use service_sc::obj::ScObjService;
use service_sc::store::{ScStoreService};
use std::cmp::{max, min};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info, warn};
use client_tg::{tg_alert, tg_msg};

pub struct ValidateSyncHandler {
    validation_repo: Arc<ValidationRepo>,
    store_service: Arc<ScStoreService>,
    validator: Arc<AndroidValidator>,
    validation_case: Arc<ValidationCase>,
}

impl ValidateSyncHandler {

    pub fn new(
        validation_repo: Arc<ValidationRepo>,
        store_service: Arc<ScStoreService>,
        validator: Arc<AndroidValidator>,
        validation_case: Arc<ValidationCase>,
    ) -> Self {
        Self { validation_repo, store_service, validator, validation_case }
    }

    // TODO v2 check from/to again before moving next
    // TODO v2 parallel
    pub async fn handle(&self, ctx: Arc<ValidationContext>) {
        let from = match self.store_service.least_request_id_to_finalize().await {
            Ok(from) => from,
            Err(e) => {
                error!("[VALIDATE_SYNC] Failed to get last validation request id. Error: {}.", e);
                tg_msg!(format!("[VALIDATE_SYNC] Failed to get last validation request id. Error: {}.", e));

                ctx.queue.async_shutdown()
                    .await;
                
                return;
            }
        };

        let to = match self.validation_case.next_validate_request_id(from).await {
            Ok(to) => to.unwrap_or_else(|| from),
            Err(err) => {
                error!("[VALIDATE_SYNC] Failed to get to value. Error: {}.", err);
                tg_alert!(format!("[VALIDATE_SYNC] Failed to get to value. Error: {}.", err));

                ctx.queue.async_shutdown()
                    .await;
                
                return;
            }
        };

        let mut tryer = SyncTrier::new(5, 1.0, 2);
        
        for request_id in from..to {
            while tryer.iterate().await {
                if ctx.queue.is_shutdown() {
                    warn!("[VALIDATE_SYNC] Validation shutting down for validator!");
                    return;
                }

                let is_validated = self.validation_repo.has_request(request_id).await;
                if is_validated {
                    break;
                }

                let request_result = self.store_service.get_request(request_id).await;
                let info = match request_result {
                    Ok(req) => req,
                    Err(err) => {
                        error!("[VALIDATE_SYNC] Failed to get request {}. Error: {}. Retrying...", request_id, err);
                        tg_msg!(format!("[VALIDATE_SYNC] Failed to get request {}. Error: {}. Retrying...", request_id, err));
                        continue; 
                    }
                };

                self.validator.validate_request(
                    info.req_type, info.target, request_id, info.data.0.as_ref()
                ).await;

                info!("[VALIDATE_SYNC] Successfully validated request id: {}", request_id);
                tg_msg!(format!("[VALIDATE_SYNC] Successfully validated request id: {}", request_id));
                break; 
            }

            if tryer.is_exceeded() {
                error!("[VALIDATE_SYNC] Failed to validate request {} after multiple retries. Moving to next ID if any.", request_id);
                tg_alert!(format!("[VALIDATE_SYNC] Failed to validate request {} after multiple retries. Moving to next ID if any.", request_id));

                ctx.queue.async_shutdown()
                    .await;
                
                return;
            }

            tryer.reset(); // TODO why?
        }
        
        ctx.queue.push_sequential(ValidatorEvent::Register)
            .await;
    }
}

use std::sync::Arc;
use core_std::trier::SyncTrier;
use service_sc::store::{ScStoreService, BlockState};
use crate::data::validation_repo::ValidationRepo;
use alloy::primitives::Address;
use tracing::{error, info};
use hex;
use client_tg::{tg_alert, tg_msg};
use core_std::hexer;
// Added for hex::encode
use crate::launcher::{ValidationContext, ValidatorEvent}; // Added ValidatorEvent

pub struct FinalizeHandler {
    validator_address: Address,
    persist: Arc<ValidationRepo>,
    service: Arc<ScStoreService>,
}

impl FinalizeHandler {
    pub fn new(
        validator_address: Address,
        persist: Arc<ValidationRepo>,
        service: Arc<ScStoreService>,
    ) -> Self {
        Self { service, persist, validator_address }
    }

    pub async fn handle(&self, build_version: u64, ctx: Arc<ValidationContext>) {
        let mut tryer = SyncTrier::new(30, 1.0, 10_000);

        while tryer.iterate().await {
            match self.service.finalize(build_version).await {
                Ok(tx) => {
                    info!("[FINALIZE_HANDLER] Finalized block {}. Tx: {}", build_version, hexer::encode_upper_pref(tx.transaction_hash));
                    tg_msg!(format!("[FINALIZE_HANDLER] Finalized block {}. Tx: {}", build_version, hexer::encode_upper_pref(tx.transaction_hash)));

                    if let Err(e) = self.persist.update_block_state(build_version, BlockState::Finalized).await {
                        error!("[FINALIZE_HANDLER] Failed to update block state to Finalized for {}: {} (but SC finalize succeeded)", build_version, e);
                    }

                    break;
                }

                Err(err) => {
                    error!("[FINALIZE_HANDLER] Attempt to finalize block {} failed. Error: {}. Checking current block state...", build_version, err);

                    match self.service.block_state(build_version, self.validator_address).await {
                        Ok(block_state_val) => {
                            if block_state_val == BlockState::Finalized {
                                info!("[FINALIZE_HANDLER] Block {} is already finalized. Proceeding.", build_version);
                                tg_msg!(format!("[FINALIZE_HANDLER] Block {} is already finalized. Proceeding.", build_version));

                                if let Err(e) = self.persist.update_block_state(build_version, BlockState::Finalized).await {
                                     error!("[FINALIZE_HANDLER] Failed to update block state for already finalized block {}: {}", build_version, e);
                                }

                                break;
                            } else {
                                error!("[FINALIZE_HANDLER] Block {} not yet finalized, state is {:?}. Retrying finalize operation...", build_version, block_state_val);
                                tg_alert!(format!("[FINALIZE_HANDLER] Block {} not yet finalized, state is {:?}. Retrying finalize operation...", build_version, block_state_val));
                            }
                        }
                        Err(state_err) => {
                            error!("[FINALIZE_HANDLER] Failed to get block state for {} after finalize attempt failed. State error: {}. Original finalize error: {}. Retrying finalize...", build_version, state_err, err);
                            tg_alert!(format!("[FINALIZE_HANDLER] Failed to get block state for {} after finalize attempt failed. State error: {}. Original finalize error: {}. Retrying finalize...", build_version, state_err, err));
                        }
                    }
                }
            }
        }

        if tryer.is_exceeded() {
            error!("[FINALIZE_HANDLER] Failed to finalize block {} after multiple retries.", build_version);
            return;
        }

        info!("[FINALIZE_HANDLER] Pushing Enqueue event after finalizing block {}.", build_version);
        ctx.queue.push(ValidatorEvent::TryAssign)
            .await;
    }
}

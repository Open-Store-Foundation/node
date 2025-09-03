use crate::launcher::{ValidationContext, ValidatorEvent};
use crate::result::ValidatorResult;
use service_sc::store::ScStoreService;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};
use client_tg::tg_msg;
use core_std::trier::SyncTrier;

pub enum FinalizationStatus {
    NotNext,
    Voting,
    Finalizing,
    Finalized,
}

pub struct ObserveVotingHandler {
    service: Arc<ScStoreService>,
}

impl ObserveVotingHandler {
    pub fn new(service: Arc<ScStoreService>) -> Self {
        Self {
            service,
        }
    }
}

impl ObserveVotingHandler {
    
    pub async fn handle(&self, block_id: u64, ctx: Arc<ValidationContext>) {
        // TODO v2 get windows from SC
        let mut tryer = SyncTrier::new(30, 1.0, 100);

        loop {
            if ctx.queue.is_shutdown() {
                warn!("[VOTE_OBSERVE] Observe event shutting down for validator!");
                return;
            }

            // TODO v2 check if block is my
            match self.finalization_status(block_id).await {
                Ok(result) => {
                    match result {
                        FinalizationStatus::NotNext => {}
                        FinalizationStatus::Voting => {}
                        FinalizationStatus::Finalizing => {
                            info!("[VOTE_OBSERVE] Voting complete");
                            ctx.queue.push(ValidatorEvent::finalize(block_id))
                                .await;

                            break;
                        }
                        FinalizationStatus::Finalized => {
                            info!("[VOTE_OBSERVE] Finalization complete for block - {}", block_id);
                            tg_msg!(format!("[VOTE_OBSERVE] Finalization complete for block - {}", block_id));
                            ctx.queue.push(ValidatorEvent::TryAssign)
                                .await;

                            break;
                        }
                    }
                },
                Err(e) => {
                    error!("[VOTE_OBSERVE] Error during finalization check for validator: {}", e);
                    tg_msg!(format!("[VOTE_OBSERVE] Error during finalization check for validator: {}", e));
                }
            }

            if !tryer.iterate().await {
                break
            }
        }
    }

    async fn finalization_status(&self, block_id: u64) -> ValidatorResult<FinalizationStatus> {
        let next_final_block = self.service.next_block_id_to_finalize()
            .await?;

        if next_final_block > block_id {
            return Ok(FinalizationStatus::Finalized);
        }

        if block_id < next_final_block {
            return Ok(FinalizationStatus::NotNext);
        }

        let result = self.service.is_finalazible(block_id)
            .await;

        if let Ok(is_finalazible) = result {
            if is_finalazible {
                return Ok(FinalizationStatus::Finalizing);
            }
        }

        Ok(FinalizationStatus::Voting)
    }
}

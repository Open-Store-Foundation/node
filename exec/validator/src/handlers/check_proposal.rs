use crate::launcher::{ValidationContext, ValidatorEvent};
use alloy::eips::BlockId;
use alloy::primitives::Address;
use core_std::trier::SyncTrier;
use service_sc::store::ScStoreService;
use std::sync::Arc;
use tracing::{error, info};
use client_tg::{tg_alert, tg_msg};

pub struct CheckProposalHandler {
    validator_address: Address,
    service: Arc<ScStoreService>
}

impl CheckProposalHandler {

    pub fn new(validator_address: Address, service: Arc<ScStoreService>) -> Self {
        Self {
            validator_address,
            service
        }
    }

    pub async fn handle(&self, block_id: Option<u64>, ctx: Arc<ValidationContext>) {
        let state = self.service.get_state(self.validator_address)
            .await;

        match state {
            Ok(state) => {
                // TODO v2 optimize
                if state.should_create_proposal() {
                    info!("[CHECK_PROPOSAL] Propose block: {}", state.my_block);
                    tg_msg!(format!("[CHECK_PROPOSAL] Propose block: {}", state.my_block));

                    ctx.queue.push(ValidatorEvent::propose(state.my_block, state.next_proposal_request))
                        .await;
                }

                if let Some(block_id) = block_id {
                    if block_id < state.next_final_block {
                        info!("[CHECK_PROPOSAL] Block already created: {}", block_id);
                        tg_msg!(format!("[CHECK_PROPOSAL] Block already created: {}", block_id));
                        return;
                    }
                    
                    if !state.is_my_block(block_id) {
                        // TODO check if there is race condition when I can vote for my own block
                        info!("[CHECK_PROPOSAL] Voting for block: {}", block_id);
                        tg_msg!(format!("[CHECK_PROPOSAL] Voting for block: {}", block_id));

                        ctx.queue.push(ValidatorEvent::vote(block_id))
                            .await;
                    }
                }
            }
            Err(e) => {
                error!("[CHECK_PROPOSAL] Failed to get state: {}", e);
                tg_msg!(format!("[CHECK_PROPOSAL] Failed to get state: {}", e));
            }
        }
    }
}

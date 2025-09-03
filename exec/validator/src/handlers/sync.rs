use crate::launcher::{ValidationAction, ValidationContext, ValidatorEvent};
use crate::result::ValidatorResult;
use alloy::primitives::Address;
use core_actor::Action;
use service_sc::store::ScStoreService;
use std::sync::Arc;
use tracing::{error, info};
use client_tg::{tg_alert, tg_msg};

pub struct SyncHandler {
    validator_address: Address,
    store_service: Arc<ScStoreService>,
}

impl SyncHandler {
    
    pub fn new(validator_address: Address, remote_storage: Arc<ScStoreService>) -> Self {
        Self {
            store_service: remote_storage,
            validator_address,
        }
    }

    // TODO v2 add retry?
    pub async fn handle(&self, ctx: Arc<ValidationContext>) {
        info!("[SYNC_HANDLER] Syncing validator state.");
        
        let state = match self.store_service.get_state(self.validator_address).await {
            Ok(state) => state,
            Err(e) => {
                error!("[SYNC_HANDLER] Failed to get state: {}. Skipping sync actions.", e);
                tg_alert!("[SYNC_HANDLER] Failed to get state!");

                ctx.queue.push_sequential(ValidatorEvent::Unregister)
                    .await;

                return;
            }
        };

        let mut actions: Vec<ValidationAction> = Vec::new();
        
        if state.can_assign_validator() {
            let result = self.store_service.can_assign_validator(self.validator_address)
                .await;

            match result {
                Ok(result) => {
                    if result {
                        actions.push(Action::new(ValidatorEvent::TryAssign));
                    }
                }
                Err(e) => {
                    // will try again later after any vote/finalization etc
                    error!("[SYNC_HANDLER] Failed to check validator assignment: {}.", e);
                    tg_alert!(format!("[SYNC_HANDLER] Failed to check validator assignment: {}.", e));
                }
            }
        }

        if state.should_create_proposal() {
            actions.push(
                Action::new(ValidatorEvent::propose(state.my_block, state.next_proposal_request))
            );
        }

        for block_id in state.next_final_block..state.next_proposal_block {
            if !state.is_my_block(block_id) {
                actions.push(Action::new(ValidatorEvent::vote(block_id)));
            } else if state.is_my_next_finalization_block(block_id) {
                actions.push(Action::parallel(ValidatorEvent::observe(block_id)));
            }
        }

        actions.push(
            Action::parallel(ValidatorEvent::poll(state.block_number))
        );
        
        info!("[SYNC_HANDLER] Synced validator state. Pushing {} actions.", actions.len());
        tg_msg!(format!("[SYNC_HANDLER] Synced validator state. Pushing {} actions.", actions.len()));
        for action in actions {
            ctx.queue.push_action(action)
                .await;
        }
    }
}

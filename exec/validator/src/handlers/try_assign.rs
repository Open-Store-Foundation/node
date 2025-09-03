use std::fmt::format;
use crate::handlers::common::top_up::TopUpCase;
use core_std::trier::SyncTrier;
use alloy::primitives::Address;
use service_sc::store::{ScStoreService, ValidatorAssignStatus};
use std::sync::Arc;
use tokio::time::sleep;
use tracing::{error, info};
use client_tg::{tg_alert, tg_msg};
use crate::launcher::{ValidationContext, ValidatorEvent};

enum AssignStage {
    CheckEligibility,
    TopUp,
    Enqueue,
}

pub struct TryAssignHandler {
    validator_address: Address,
    service: Arc<ScStoreService>,
    top_up: Arc<TopUpCase>,
}

impl TryAssignHandler {

    pub fn new(
        validator_address: Address,
        service: Arc<ScStoreService>,
        handler: Arc<TopUpCase>,
    ) -> Self {
        Self {
            validator_address,
            service,
            top_up: handler,
        }
    }

    pub async fn handle(&self, ctx: Arc<ValidationContext>) {
        info!("[ASSIGN_HANDLER] Start assigning...");

        let mut trier = SyncTrier::new(30, 1.0, 2);
        let mut stage = AssignStage::CheckEligibility;
        let mut block_id  = 0u64;

        'trier: while trier.iterate().await {
            loop {
                stage = match stage {
                    AssignStage::CheckEligibility => {
                        let result = self.service.validator_assign_status(self.validator_address)
                            .await;

                        let status = match result {
                            Ok(status) => status,
                            Err(err) => {
                                error!("[ASSIGN_HANDLER] Can't check if registered. Error: {}", err);
                                tg_alert!(format!("[ASSIGN_HANDLER] Can't check if registered. Error: {}", err));
                                continue 'trier;
                            }
                        };

                        match status {
                            ValidatorAssignStatus::ValidatorVersionOutdated => {
                                info!("[ASSIGN_HANDLER] Validator's version is too low!");
                                tg_alert!("[ASSIGN_HANDLER] Validator's version is too low!");
                                ctx.queue.push(ValidatorEvent::Unregister)
                                    .await;

                                break 'trier;
                            }
                            ValidatorAssignStatus::NotRegistered => {
                                info!("[ASSIGN_HANDLER] Validator is not registered!");
                                tg_msg!("[ASSIGN_HANDLER] Validator is not registered!");
                                ctx.queue.push(ValidatorEvent::Register)
                                    .await;

                                break 'trier;
                            }
                            ValidatorAssignStatus::NotEnoughVotes => {
                                error!("[ASSIGN_HANDLER] Validator is not assignable on the block you need more voteBalance!");
                                tg_alert!("[ASSIGN_HANDLER] Validator is not registered!");
                                break 'trier;
                            }
                            ValidatorAssignStatus::AlreadyAssigned => {
                                error!("[ASSIGN_HANDLER] Validator already have assigned block: {}!", block_id);
                                tg_msg!(format!("[ASSIGN_HANDLER] Validator already have assigned block: {}!", block_id));
                                break 'trier;
                            }
                            ValidatorAssignStatus::Assignable => {
                                AssignStage::TopUp
                            }
                        }
                    },
                    AssignStage::TopUp => {
                        if !self.top_up.handle().await {
                            continue 'trier;
                        }

                        AssignStage::Enqueue
                    },
                    AssignStage::Enqueue => {
                        let result = self.service.next_assign_block_id()
                            .await;

                        let next_block = match result {
                            Ok(val) => val,
                            Err(err) => {
                                error!("[ASSIGN_HANDLER] Can't get next enqueue block id. Error: {}", err);
                                tg_alert!(format!("[ASSIGN_HANDLER] Can't get next enqueue block id. Error: {}", err));
                                continue 'trier;
                            }
                        };

                        let result = self.service.assign_validator(next_block)
                            .await;

                        if let Err(err) = result {
                            let result = self.service.next_block_id_for(self.validator_address)
                                .await;

                            match result {
                                Ok(next_block) if next_block > 0 => {
                                    block_id = next_block;
                                    info!("[ASSIGN_HANDLER] Successfully enqueued validator {} at block {}.", self.validator_address, next_block);
                                    tg_msg!(format!("[ASSIGN_HANDLER] Successfully enqueued validator {} at block {}.", self.validator_address, next_block));
                                    break 'trier;
                                }
                                _ => {
                                    error!("[ASSIGN_HANDLER] Can't enqueue validator. Error: {}", err);
                                    tg_alert!(format!("[ASSIGN_HANDLER] Can't enqueue validator. Error: {}", err));

                                    continue 'trier;
                                }
                            }
                        };

                        block_id = next_block;
                        info!("[ASSIGN_HANDLER] Successfully enqueued validator {} at block {}.", self.validator_address, next_block);
                        tg_msg!(format!("[ASSIGN_HANDLER] Successfully enqueued validator {} at block {}.", self.validator_address, next_block));

                        break 'trier;
                    },
                };
            }
        }

        if trier.is_failed() {
            // Will try next time after voting or some other operation
            error!("[ASSIGN_HANDLER] Enqueue enqueued validators failed!");
            tg_alert!("[ASSIGN_HANDLER] Enqueue enqueued validators failed!");
        }

        if block_id > 0 {
            ctx.queue.push(ValidatorEvent::check_proposal(Some(block_id)))
                .await
        }
    }
}

use crate::data::block_repo::BlockRepo;
use crate::data::validation_repo::ValidationRepo;
use crate::handlers::common::create_proposal::{CreateProposalCase, ProposeBlockContext};
use crate::handlers::common::top_up::TopUpCase;
use crate::handlers::common::validation::ValidationCase;
use crate::handlers::validate_sync::ValidateSyncHandler;
use crate::launcher::{ValidationContext, ValidatorEvent};
use crate::result::ValidatorResult;
use crate::utils::stage::Stage;
use alloy::primitives::Address;
use codegen_block::block::{ValidationBlock, ValidationResult};
use core_std::trier::SyncTrier;
use openssl::sha::sha256;
use prost::Message;
use service_sc::store::{BlockState, ScStoreService};
use std::cmp::min;
use std::sync::Arc;
use tracing::{error, info, warn};
use client_tg::{tg_alert, tg_msg};

enum ProposalStage {
    Prepare,
    ProposePoll(ProposeBlockContext),
}

pub enum AssignedStatus {
    Assigned,
    Submitted,
    Unknown,
}

pub struct ProposeHandler {
    validator_address: Address,
    persist: Arc<ValidationRepo>,
    service: Arc<ScStoreService>,
    propose_case: Arc<CreateProposalCase>,
    validation_case: Arc<ValidationCase>,
}

// TODO v2 make configurable
const MAX_TRY_PER_REVALIDATE: u32 = 5;

impl ProposeHandler {

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        validator_address: Address,
        persist: Arc<ValidationRepo>,
        service: Arc<ScStoreService>,
        propose_case: Arc<CreateProposalCase>,
        validation_case: Arc<ValidationCase>,
    ) -> Self {
        Self {
            persist,
            service,
            validator_address,
            propose_case,
            validation_case
        }
    }

    pub async fn handle(&self, block_id: u64, from: u64, ctx: Arc<ValidationContext>) {
        let mut tryer = SyncTrier::new(30, 1.0, 1000);

        let mut stage = Stage::Checking(None);
        let mut block_stage = None;
        let mut is_proposal_created = false;

        'looper: loop {
            if ctx.queue.is_shutdown() { // TODO v2 add parameter if we need finish
                return;
            }

            stage = match stage {
                Stage::Retry(stage) => {
                    if !tryer.iterate().await {
                        break 'looper;
                    }

                    match stage {
                        Some(value) => Stage::Value(value),
                        None => Stage::check_none(),
                    }
                }
                Stage::Checking(stage) => {
                    match self.assigned_state(block_id).await {
                        Ok(state) => match state {
                            AssignedStatus::Assigned => {
                                Stage::Value(stage.unwrap_or_else(|| ProposalStage::Prepare))
                            }
                            AssignedStatus::Submitted => {
                                info!("[PROPOSE_HANDLER] Block {} already submitted or processed.", block_id);
                                tg_msg!(format!("[PROPOSE_HANDLER] Block {} already submitted or processed.", block_id));
                                is_proposal_created = true;
                                break 'looper;
                            }
                            AssignedStatus::Unknown => {
                                info!("[PROPOSE_HANDLER] Block {} is not assigned on you!.", block_id);
                                tg_msg!(format!("[PROPOSE_HANDLER] Block {} is not assigned on you!.", block_id));
                                break 'looper;
                            }
                        },
                        Err(e) => {
                            error!("[PROPOSE_HANDLER] Error checking if block {} is submitted: {}. Retrying...", block_id, e);
                            tg_alert!(format!("[PROPOSE_HANDLER] Error checking if block {} is submitted: {}. Retrying...", block_id, e));
                            match stage {
                                Some(value) => Stage::retry(value),
                                None => Stage::retry_none(),
                            }
                        }
                    }
                }
                Stage::Value(value) => match value {
                    ProposalStage::Prepare => {
                        let data = self.validation_case.validation_block(block_id, from, None)
                            .await;

                        match data {
                            Ok(data) => match data {
                                Some(block) => {
                                    Stage::Value(ProposalStage::ProposePoll(ProposeBlockContext::proposal(block)))
                                }
                                None => {
                                    error!("[PROPOSE_HANDLER] Can't get any requests to validate. Skip handle.");
                                    tg_alert!("[PROPOSE_HANDLER] Can't get any requests to validate. Skip handle.");
                                    break 'looper;
                                }
                            },
                            Err(err) => {
                                error!("[PROPOSE_HANDLER] Can't get last validated request. Error: {}.", err);
                                tg_alert!(format!("[PROPOSE_HANDLER] Can't get last validated request. Error: {}.", err));
                                if tryer.try_count() % MAX_TRY_PER_REVALIDATE == 0 {
                                    Stage::check_none()
                                } else {
                                    Stage::retry(ProposalStage::Prepare)
                                }
                            }
                        }
                    }
                    // TODO v2 refactor inner poll
                    ProposalStage::ProposePoll(ctx) => {
                        block_stage = self.propose_case.poll(block_stage, &ctx)
                            .await;

                        match block_stage {
                            None => {
                                info!("[PROPOSE_HANDLER] Proposal for block {} successfully created.", block_id);
                                tg_msg!(format!("[PROPOSE_HANDLER] Proposal for block {} successfully created.", block_id));
                                is_proposal_created = true;
                                break 'looper;
                            }
                            Some(_) => {
                                info!("[PROPOSE_HANDLER] Proposal for block {} is not ready yet. Retrying...", block_id);
                                tg_msg!(format!("[PROPOSE_HANDLER] Proposal for block {} is not ready yet. Retrying...", block_id));
                                // logging inside
                                if tryer.try_count() % MAX_TRY_PER_REVALIDATE == 0 {
                                    Stage::check(ProposalStage::ProposePoll(ctx))
                                } else {
                                    Stage::retry(ProposalStage::ProposePoll(ctx))
                                }
                            }
                        }
                    }
                }
            };
        }

        if tryer.is_exceeded() {
            error!("[PROPOSE_HANDLER] Failed to propose block {} after multiple retries.", block_id);
            tg_alert!(format!("[PROPOSE_HANDLER] Failed to propose block {} after multiple retries.", block_id));
        }

        if is_proposal_created {
            ctx.queue.push(ValidatorEvent::observe(block_id))
                .await;
        }
    }

    pub async fn assigned_state(&self, block_id: u64) -> ValidatorResult<AssignedStatus> {
        if self.persist.is_submitted(block_id).await? {
            return Ok(AssignedStatus::Submitted);
        }

        info!("[PROPOSE_HANDLER] Can't find block state locally, fetching from remote!");
        let block_state_val = self.service.block_state(block_id, self.validator_address).await?;

        info!("[PROPOSE_HANDLER] Remote block state - {block_state_val}!");
        if block_state_val.at_least_proposed() {
            let _ = self.persist.update_block_state(block_id, block_state_val);
            return Ok(AssignedStatus::Submitted);
        }

        if block_state_val.at_least_assigned() {
            return Ok(AssignedStatus::Assigned);
        }

        Ok(AssignedStatus::Unknown)
    }
}

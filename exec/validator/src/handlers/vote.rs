use crate::data::validation_repo::ValidationRepo;
use crate::ext::validation_block::proto_sha256;
use crate::handlers::common::create_proposal::{CreateProposalCase, ProposeBlockContext};
use crate::handlers::common::validation::ValidationCase;
use crate::launcher::{ValidationContext, ValidatorEvent};
use crate::result::ValidatorResult;
use crate::utils::stage::Stage;
use alloy::hex::FromHex;
use alloy::primitives::{Address, TxHash, U256};
use codegen_block::block::ValidationBlock;
use codegen_contracts::ext::{read_2bit_status, write_2bit_status};
use core_std::trier::SyncTrier;
use prost::Message;
use service_sc::store::{BlockState, ScStoreService, StoreBlockRef};
use std::sync::Arc;
use std::u128;
use tracing::{error, info, warn};
use client_tg::{tg_alert, tg_msg};
use net_client::node::result::EthResult;

enum VotingStage {
    BlockInfo(Option<usize>),
    BlockData(StoreBlockRef, Option<usize>),
    Validate(StoreBlockRef, Option<ValidationBlock>, Option<usize>),
    Discussion(ProposeBlockContext),
    Vote(StoreBlockRef, u128),
}

pub enum VoteStatus {
    Assigned,
    Proposed,
    Voted,
    Discussing,
    Unknown,
}

pub struct VoteHandler {
    validator_address: Address,
    persist: Arc<ValidationRepo>,
    service: Arc<ScStoreService>,
    propose_case: Arc<CreateProposalCase>,
    validation_case: Arc<ValidationCase>,
}

const MAX_TRY_PER_REVALIDATE: u32 = 5;

impl VoteHandler {

    pub fn new(
        validator_address: Address,
        persist: Arc<ValidationRepo>,
        service: Arc<ScStoreService>,
        propose_case: Arc<CreateProposalCase>,
        validation_case: Arc<ValidationCase>,
    ) -> Self {
        Self {
            validator_address,
            persist,
            service,
            propose_case,
            validation_case,
        }
    }

    // TODO discuss 2 block when you already created 1 and you disagree with another
    pub async fn handle(&self, block_id: u64, ctx: Arc<ValidationContext>) {
        let mut tryer = SyncTrier::new(30, 1.0, 2);

        let mut stage = Stage::Checking(None);
        let mut block_stage = None;
        let mut is_voted = false;
        let mut is_discussed = false;

        if ctx.queue.is_shutdown() {
            warn!("[VOTE_HANDLER] Voting event shutting down for validator!");
            tg_msg!("[VOTE_HANDLER] Voting event shutting down for validator!");
            return;
        }
        
        'looper: loop {
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
                    match self.voted_state(block_id).await {
                        Ok(state) => match state {
                            VoteStatus::Assigned => {
                                warn!("[VOTE_HANDLER] Block {} is assigned. Please investigate, unexpected behaviour!", block_id);
                                tg_msg!(format!("[VOTE_HANDLER] Block {} is assigned. Please investigate, unexpected behaviour!", block_id));

                                ctx.queue.push(ValidatorEvent::check_proposal(Some(block_id)))
                                    .await;

                                return;
                            }
                            VoteStatus::Proposed => {
                                Stage::Value(stage.unwrap_or_else(|| VotingStage::BlockInfo(None)))
                            },
                            VoteStatus::Voted => {
                                info!("[VOTE_HANDLER] Block {} already submitted.", block_id);
                                tg_msg!(format!("[VOTE_HANDLER] Block {} already submitted.", block_id));
                                break 'looper;
                            }
                            VoteStatus::Discussing => {
                                info!("[VOTE_HANDLER] Block {} is discussing.", block_id);
                                tg_msg!(format!("[VOTE_HANDLER] Block {} is discussing.", block_id));
                                is_discussed = true;
                                break 'looper;
                            }
                            VoteStatus::Unknown => {
                                info!("[VOTE_HANDLER] Block {} is not assigned on you!.", block_id);
                                tg_msg!(format!("[VOTE_HANDLER] Block {} is not assigned on you!.", block_id));
                                break 'looper;
                            }
                        },
                        Err(e) => {
                            error!("[VOTE_HANDLER] Error checking if block {} is submitted: {}. Retrying...", block_id, e);
                            tg_alert!(format!("[VOTE_HANDLER] Error checking if block {} is submitted: {}. Retrying...", block_id, e));
                            match stage {
                                Some(value) => Stage::retry(value),
                                None => Stage::retry_none(),
                            }
                        }
                    }
                }
                Stage::Value(value) => match value {
                    VotingStage::BlockInfo(proposer_id) => {
                        let next_id = proposer_id.clone().map_or(0, |id| id + 1);
                        let proposers = match self.service.get_block_proposers(block_id).await {
                            Ok(proposers) => proposers,
                            Err(err) => {
                                error!("[VOTE_HANDLER] Can't get block info for {}. Error: {}. Retrying...", block_id, err);
                                tg_alert!(format!("[VOTE_HANDLER] Can't get block info for {}. Error: {}. Retrying...", block_id, err));
                                stage = Stage::retry(VotingStage::BlockInfo(proposer_id));
                                continue
                            }
                        };
                        
                        if proposers.is_empty() {
                            warn!("[VOTE_HANDLER] No proposers for block {}. Skip handle.", block_id);
                            tg_msg!(format!("[VOTE_HANDLER] No proposers for block {}. Skip handle.", block_id));
                            return;
                        }
                        
                        let next_proposer  = proposers.get(next_id);
                        let proposer = match next_proposer {
                            Some(next_proposer) => next_proposer.clone(),
                            None => match proposers.first() {
                                Some(next_proposer) => next_proposer.clone(),
                                None => {
                                    warn!("[VOTE_HANDLER] No proposers for block {}. Skip handle.", block_id);
                                    tg_msg!(format!("[VOTE_HANDLER] No proposers for block {}. Skip handle.", block_id));
                                    return;
                                }
                            }
                        };
                            
                        match self.service.get_block_info(block_id, proposer).await {
                            Ok(info) => match next_proposer {
                                Some(_) => Stage::Value(VotingStage::BlockData(info, Some(next_id))),
                                None => Stage::Value(VotingStage::Validate(info, None, proposer_id))
                            },
                            Err(err) => {
                                error!("[VOTE_HANDLER] Can't get block info for {}. Error: {}. Retrying...", block_id, err);
                                tg_alert!(format!("[VOTE_HANDLER] Can't get block info for {}. Error: {}. Retrying...", block_id, err));
                                Stage::Retry(Some(VotingStage::BlockInfo(proposer_id)))
                            }
                        }
                    }
                    VotingStage::BlockData(info, proposer) => {
                        let tx = match TxHash::from_hex(info.ref_id.as_slice()) {
                            Ok(hash) => hash,
                            Err(err) => {
                                error!("[VOTE_HANDLER] Can't get block info for {}. Error: {}. Retrying...", block_id, err);
                                tg_alert!(format!("[VOTE_HANDLER] Can't get block info for {}. Error: {}. Retrying...", block_id, err));
                                stage = Stage::Value(VotingStage::BlockInfo(proposer));
                                continue
                            }
                        };

                        let data = match self.service.get_block_data(tx).await {
                            Ok(data) => data,
                            Err(err) => {
                                error!("[VOTE_HANDLER] Can't get block data for {}. Error: {}. Retrying...", block_id, err);
                                tg_alert!(format!("[VOTE_HANDLER] Can't get block data for {}. Error: {}. Retrying...", block_id, err));
                                stage = Stage::Retry(Some(VotingStage::BlockData(info, proposer)));
                                continue
                            }
                        };

                        match data {
                            Some(data) => {
                                match ValidationBlock::decode(data.as_slice()) {
                                    Ok(block) => {
                                        Stage::Value(VotingStage::Validate(info, Some(block), proposer))
                                    },
                                    Err(_) => {
                                        error!("[VOTE_HANDLER] Can't decode block data for {}", block_id);
                                        tg_alert!(format!("[VOTE_HANDLER] Can't decode block data for {}", block_id));
                                        Stage::Value(VotingStage::BlockInfo(proposer))
                                    }
                                }
                            }
                            None => {
                                error!("[VOTE_HANDLER] Can't get block data for {}.", block_id);
                                tg_alert!(format!("[VOTE_HANDLER] Can't get block data for {}.", block_id));
                                Stage::Value(VotingStage::BlockInfo(proposer))
                            }
                        }
                    }
                    VotingStage::Validate(validating_info, validation_block, proposer) => {
                        let from = validating_info.from_request_id;
                        let to = validating_info.to_request_id;
                        info!("[VOTE_HANDLER] Validating requests from {} to {} for voting on block {}.",from, to, block_id);
                        tg_msg!(format!("[VOTE_HANDLER] Validating requests from {} to {} for voting on block {}.",from, to, block_id));

                        let data = self.validation_case
                            .validation_block(block_id, from, Some(to))
                            .await;

                        match data {
                            Ok(data) => match data {
                                Some(own_block) => {
                                    let validating_block = match validation_block {
                                        Some(validation_block) => validation_block,
                                        None => {
                                            let ctx = ProposeBlockContext::discussion(own_block);
                                            stage = Stage::Value(VotingStage::Discussion(ctx));
                                            continue;
                                        }
                                    };

                                    if !self.is_valid_block_data(&validating_info, &validating_block) {
                                        stage = Stage::Value(VotingStage::BlockInfo(proposer));
                                        continue;
                                    }

                                    let (aligned, mask) = self.align_blocks(&own_block, &validating_block);
                                    let ctx = ProposeBlockContext::discussion(aligned);
                                    // TODO if file hash has changed - we should put unavailable
                                    // TODO add hash to AppBuild?
                                    if ctx.block_hash.as_slice() == validating_info.block_hash.as_slice() {
                                        Stage::Value(VotingStage::Vote(validating_info, mask))
                                    } else {
                                        let ctx = ProposeBlockContext::discussion(own_block);
                                        Stage::Value(VotingStage::Discussion(ctx))
                                    }
                                }
                                None => {
                                    error!("[VOTE_HANDLER] Can't get any requests to validate. Skip handle.");
                                    tg_alert!("[VOTE_HANDLER] Can't get any requests to validate. Skip handle.");
                                    break 'looper;
                                }
                            },
                            Err(err) => {
                                error!("[VOTE_HANDLER] Can't get last validated request. Error: {}.", err);
                                tg_alert!(format!("[VOTE_HANDLER] Can't get last validated request. Error: {}.", err));

                                if tryer.retry_count() % MAX_TRY_PER_REVALIDATE == 0 {
                                    Stage::check(VotingStage::Validate(validating_info, validation_block, proposer))
                                } else {
                                    Stage::retry(VotingStage::Validate(validating_info, validation_block, proposer))
                                }
                            }
                        }
                    }
                    VotingStage::Vote(info, mask) => {
                        info!("[VOTE_HANDLER] Object hash matches for block {}. Voting.",block_id);
                        let result = self.service.vote(block_id, self.validator_address, mask)
                            .await;

                        match result {
                            Ok(_) => {
                                let p_result = self.persist
                                    .update_block_state(block_id, BlockState::Discussing)
                                    .await;

                                if let Err(e) = p_result {
                                    error!("[VOTE_HANDLER] Failed to update block state to Voted for {}: {} but proposal created.", block_id, e);
                                    tg_alert!(format!("[VOTE_HANDLER] Failed to update block state to Voted for {}: {} but proposal created.", block_id, e));
                                }

                                is_voted = true;
                                break 'looper;
                            }
                            Err(e) => {
                                error!("[VOTE_HANDLER] Failed to vote for block {}: {}. Retrying...", block_id, e);
                                tg_alert!(format!("[VOTE_HANDLER] Failed to vote for block {}: {}. Retrying...", block_id, e));
                                if tryer.retry_count() % MAX_TRY_PER_REVALIDATE == 0 {
                                    Stage::check(VotingStage::Vote(info, mask))
                                } else {
                                    Stage::retry(VotingStage::Vote(info, mask))
                                }
                            }
                        }
                    }
                    VotingStage::Discussion(ctx) => {
                        block_stage = self.propose_case.poll(block_stage, &ctx)
                            .await;

                        match block_stage {
                            None => {
                                info!("[VOTE_HANDLER] Discussion for block {} successfully created.", block_id);
                                tg_alert!(format!("[VOTE_HANDLER] Discussion for block {} successfully created.", block_id));

                                is_discussed = true;
                                break 'looper;
                            }
                            Some(_) => {
                                if tryer.retry_count() % MAX_TRY_PER_REVALIDATE == 0 {
                                    Stage::check(VotingStage::Discussion(ctx))
                                } else {
                                    Stage::retry(VotingStage::Discussion(ctx))
                                }
                            }
                        }
                    }
                }
            };
        }

        if tryer.is_exceeded() {
            error!("[VOTE_HANDLER] Failed to handle voting block {} after multiple retries.", block_id);
            tg_alert!(format!("[VOTE_HANDLER] Failed to handle voting block {} after multiple retries.", block_id));
        }

        if is_voted {
            ctx.queue.push(ValidatorEvent::TryAssign)
                .await;
        }

        if is_discussed {
            ctx.queue.push(ValidatorEvent::observe(block_id))
                .await;
        }
    }

    pub async fn voted_state(&self, block_id: u64) -> ValidatorResult<VoteStatus> {
        if self.persist.is_voted(block_id).await? {
            return Ok(VoteStatus::Voted);
        }

        info!("[VOTE_HANDLER] Can't find block state locally, fetching from remote!");
        tg_msg!("[VOTE_HANDLER] Can't find block state locally, fetching from remote!");
        let block_state_val = self
            .service
            .block_state(block_id, self.validator_address)
            .await?;
        info!("[VOTE_HANDLER] Remote block state - {block_state_val}!");
        tg_msg!(format!("[VOTE_HANDLER] Remote block state - {block_state_val}!"));

        if block_state_val.at_least_voted() {
            let _ = self.persist.update_block_state(block_id, block_state_val);
            return Ok(VoteStatus::Voted);
        }

        if block_state_val.is_discussing() {
            let _ = self.persist.update_block_state(block_id, block_state_val);
            return Ok(VoteStatus::Discussing);
        }

        if block_state_val.is_proposed() {
            return Ok(VoteStatus::Proposed);
        }
        
        if block_state_val.is_assigned() {
            return Ok(VoteStatus::Assigned);
        }

        Ok(VoteStatus::Unknown)
    }

    fn is_valid_block_data(&self, info: &StoreBlockRef, block: &ValidationBlock) -> bool {
        if info.id != block.id {
            return false;
        }
        
        let Some(from) = block.from_request_id() else {
            return false;       
        };
        
        let Some(to) = block.to_request_id() else {
            return false;      
        };
        
        if info.from_request_id != from {
            return false;
        }

        if info.to_request_id != to {
            return false;
        }

        let block_hash = proto_sha256(block);
        if info.block_hash.as_slice() != block_hash {
            return false;
        }

        for i in 0..block.requests.len() {
            // check is incremental
            let req = &block.requests[i];
            if req.request_id != from + i as u64 {
                return false;
            }

            // Check statues is valid
            let status = read_2bit_status(info.result, i);
            if status == 3 && req.status < 3 {
                return false;
            }
            
            if status != req.status {
                return false;
            }       
        }

        return true;
    }

    fn align_blocks(&self, block: &ValidationBlock, with: &ValidationBlock) -> (ValidationBlock, u128) {
        let mut result = block.clone();
        let mut unavailability_mask = 0u128;

        for i in 0..block.requests.len() {
            // req id should be the same with our block, we checked in is_valid_block_data
            let own = &block.requests[i];
            let validating = &with.requests[i];

            if validating.status == 0 && own.status != 0 {
                // we should have the same content to be able to compare other fields
                result.requests[i] = validating.clone();
            } else if own.status == 0 && validating.status != 0 {
                result.requests[i] = validating.clone();
                unavailability_mask |= 1 << i;
            } else if own.status == 0 {
                unavailability_mask |= 1 << i;
            }
        }

        (result, unavailability_mask)
    }
}

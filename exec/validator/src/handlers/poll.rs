use crate::android::validator::AndroidValidator;
use crate::data::validation_repo::ValidationRepo;
use crate::env;
use crate::launcher::ValidatorEvent::CheckProposal;
use crate::launcher::{ValidationAction, ValidationContext, ValidatorEvent};
use crate::result::ValidatorResult;
use alloy::primitives::{Address, B256};
use alloy::providers::Provider;
use alloy::rpc::types::{Filter, Log};
use alloy::sol_types::sol_data::{Bool, Int, Uint};
use alloy::sol_types::{sol_data, SolType};
use codegen_contracts::ext::ToChecksum;
use core_actor::Action;
use core_std::trier::SyncTrier;
use derive_more::Display;
use net_client::node::provider::Web3Provider;
use prost::Message;
use service_sc::obj::ScObjService;
use service_sc::store::ScStoreService;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};
use client_tg::{tg_alert, tg_msg};

struct PollReady {
    pub block_number: u64,
    pub events_count: usize,
}

impl PollReady {
    pub fn new(block_number: u64, events_count: usize) -> Self {
        Self { block_number, events_count }
    }
}

type PollResult<T> = Result<T, PollError>;

#[derive(Display, Debug)]
enum PollError {
    UndefinedBehaviour
}

pub struct ValidatorEventPoolConfig {
    pub filter_block_threshold: u64,
    pub topics: Vec<B256>,
    pub address: Address,
    pub timeout: Duration,
    pub dry_timeout: Duration,
}

pub struct PollHandler {
    provider: Arc<Web3Provider>,
    validator: Arc<AndroidValidator>,
    config: ValidatorEventPoolConfig,
    validation_repo: Arc<ValidationRepo>,
}

impl PollHandler {

    pub fn new(
        config: ValidatorEventPoolConfig,
        provider: Arc<Web3Provider>,
        validator: Arc<AndroidValidator>,
        validation_repo: Arc<ValidationRepo>,
    ) -> Self {
        Self {
            provider,
            validator,
            validation_repo,
            config
        }
    }
}

pub trait EvmEventPool {
    async fn spawn_polling(&self, block_number: u64, ctx: Arc<ValidationContext>);
}

impl EvmEventPool for PollHandler {

    async fn spawn_polling(&self, block_number: u64, ctx: Arc<ValidationContext>) {
        info!("[POLL] Spawn polling from block {}...", block_number);
        let mut block_pointer = block_number;
        
        loop {
            if ctx.queue.is_shutdown() {
                warn!("[POLL] Poll event shutting down!");
                return;
            }

            let result = self.poll(block_pointer, ctx.clone())
                .await;

            let poll = match result {
                Ok(poll) => poll,
                Err(e) => {
                    error!("[POLL] Failed to poll events: {}. Block: {}", e, block_pointer);
                    tg_alert!(format!("[POLL] Failed to poll events: {}. Block: {}", e, block_pointer));
                    ctx.queue.push(ValidatorEvent::Unregister)
                        .await;

                    return;
                }
            };

            block_pointer = poll.block_number;

            if poll.events_count == 0 {
                sleep(self.config.dry_timeout)
                    .await;
            } else {
                sleep(self.config.timeout)
                    .await;
            }
        }
    }
}

impl PollHandler {

    async fn poll(&self, block_number: u64, ctx: Arc<ValidationContext>) -> PollResult<PollReady> {
        info!("[POLL] Poll events from block {}...", block_number);
        let end_block = match self.provider.get_block_number().await {
            Ok(current_head) => current_head.min(block_number.saturating_add(self.config.filter_block_threshold)),
            Err(e) => {
                error!("[POLL] Error getting current block number: {}", e);
                return Ok(PollReady::new(block_number, 0));
            }
        };

        let filter = Filter::new()
            .address(self.config.address)
            .event_signature(self.config.topics.clone())
            .from_block(block_number)
            .to_block(end_block);

        let logs_result = self.provider.get_logs(&filter)
            .await;

        let logs = match logs_result {
            Ok(l) => l,
            Err(e) => {
                error!("[POLL] Failed to get logs from block {}: {}.", block_number, e);
                return Ok(PollReady::new(block_number, 0));
            }
        };

        let event_count = logs.len();

        if event_count == 0 {
            info!("[POLL] Sync events | EMPTY | From block {}!", block_number);
        } else {
            info!("[POLL] Sync events | Count {} | From block {}!", event_count, block_number);
        }

        for item in logs.iter() {
            let opt = self.handle_log(item)
                .await?;

            if let Some(action) = opt {
                ctx.queue.push(action)
                    .await;
            }
        }

        return Ok(PollReady::new(end_block + 1, event_count));
    }

    async fn handle_log(&self, item: &Log) -> PollResult<Option<ValidatorEvent>> {
        let topic = item.topics()[0];

        return match topic {
            ScStoreService::NEW_REQUEST_HASH => self.handle_new_request(item).await,
            ScStoreService::BLOCK_PROPOSED_HASH => self.handle_proposed_log(item).await,
            _ => {
                error!("[POLL] Can't handle event with unknown topic! Log: {:?}", item);
                tg_alert!(format!("[POLL] Can't handle event with unknown topic! Log: {:?}", item));
                return Ok(None)
            },
        }
    }

    async fn handle_new_request(&self, item: &Log) -> PollResult<Option<ValidatorEvent>> {
        let result = ScStoreService::decode_new_request(item.as_ref());
        let (request_type, app, request_id, data) = match result {
            Ok(result) => (result.data.reqType, result.data.target, result.data.requestId.to::<u64>(), result.data.data),
            Err(e) => {
                error!("[POLL_BUILD] Can't decode app's event data: {}. Log: {:?}", e, item);
                tg_alert!(format!("[POLL_BUILD] Can't decode app's event data: {}. Log: {:?}", e, item));
                return Err(PollError::UndefinedBehaviour);
            }
        };

        // TODO v2 stable mechanism for RPC calls
        // We don't use ValidationCase here, because we have data from Event
        // We need a stable mechanism for RPC view calls to chain to use ValidationCase
        let is_validated = self.validation_repo.has_request(request_id)
            .await;

        if is_validated {
            info!("[POLL_BUILD] Request {} already validated!", request_id);
            return Ok(None)
        }

        info!("[POLL_BUILD] Poll handle request: {}, app: {}", request_id, app.upper_checksum());
        let result = self.validator.validate_request(request_type.to(), app, request_id, data.as_ref())
            .await;
        info!("[POLL_BUILD] Request {} validation result: {}", request_id, result.status);

        return Ok(Some(ValidatorEvent::check_proposal(None)))
    }

    async fn handle_proposed_log(&self, item: &Log) -> PollResult<Option<ValidatorEvent>> {
        let result = ScStoreService::decode_proposed(item.as_ref());
        let block_id = match result {
            Ok(result) => result.blockId,
            Err(e) => {
                error!("[POLL_PROPOSAL] Can't decode app's event data: {}. Log: {:?}", e, item);
                tg_alert!(format!("[POLL_PROPOSAL] Can't decode app's event data: {}. Log: {:?}", e, item));
                return Err(PollError::UndefinedBehaviour);
            }
        };

        if let Ok(is_submitted) = self.validation_repo.is_submitted(block_id.to()).await {
            if is_submitted {
                info!("[POLL_PROPOSAL] Block {} already submitted! Skip checking!", block_id);
                return Ok(None)
            }
        }
        
        return Ok(
            Some(ValidatorEvent::check_proposal(Some(block_id.to())))
        )
    }
}


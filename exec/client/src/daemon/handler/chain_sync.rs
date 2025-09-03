use crate::daemon::handler::sync::add_to_track::AddToTrack;
use crate::daemon::handler::sync::block_finalized::BlockFinalizedHandler;
use crate::daemon::handler::sync::new_req::NewRequestHandler;
use crate::daemon::handler::sync::sync_finish::SyncFinishedHandler;
use crate::daemon::launcher::{DaemonAction, DaemonContex};
use crate::data::repo::batch_repo::{BatchRepo, TransactionBatch, TransactionStatus};
use crate::env;
use crate::result::{ClientError, ClientResult};
use alloy::hex::ToHexExt;
use alloy::primitives::{Address, B256};
use alloy::providers::Provider;
use alloy::rpc::types::{Filter, Log};
use codegen_contracts::ext::ToChecksum;
use net_client::node::provider::Web3Provider;
use net_client::node::result::EthError;
use service_ethscan::client::EthScanClient;
use service_ethscan::error::EthScanError;
use service_ethscan::models::GetLogsParams;
use service_sc::assetlinks::ScAssetLinkService;
use service_sc::store::ScStoreService;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};

pub struct ChainSyncHandler {
    store_created_block: u64,
    filter_block_threshold: u64,
    retry_timeout: Duration,
    empty_timeout: Duration,
    eth: Arc<Web3Provider>,
    ethscan: Arc<EthScanClient>,
    batch_repo: Arc<BatchRepo>,
    sync_finished: Arc<SyncFinishedHandler>,
    new_request: Arc<NewRequestHandler>,
    block_finalized: Arc<BlockFinalizedHandler>,
    add_to_track: Arc<AddToTrack>,
}

impl ChainSyncHandler {

    pub fn new(
        store_created_block: u64,
        filter_block_threshold: u64,
        retry_timeout: Duration,
        empty_timeout: Duration,
        eth: Arc<Web3Provider>,
        ethscan: Arc<EthScanClient>,
        batch_repo: Arc<BatchRepo>,
        sync_finished: Arc<SyncFinishedHandler>,
        new_request: Arc<NewRequestHandler>,
        block_finalized: Arc<BlockFinalizedHandler>,
        add_to_track: Arc<AddToTrack>,
    ) -> Self {
        Self { store_created_block, filter_block_threshold, retry_timeout, empty_timeout, eth, ethscan, batch_repo, sync_finished, block_finalized, new_request, add_to_track }
    }

    pub async fn handle(&self, ctx: Arc<DaemonContex>) {
        self.sync_logs(ctx)
            .await;
    }

    ///////////
    // RUN
    ///////////
    async fn sync_logs(&self, ctx: Arc<DaemonContex>) {
        let Ok(mut next_block_number) = self.bulk_sync().await else {
            ctx.queue.push_sequential(DaemonAction::Shutdown)
                .await;
            
            return;
        };
        
        let inclusive_block_gap = self.filter_block_threshold - 1;

        let addresses = vec![
            env::assetlink_address(),
            env::openstore_address(),
        ];

        let signatures = vec![
            ScAssetLinkService::SYNC_FINISH_HASH,
            ScStoreService::BLOCK_FINALIZED_HASH,
            ScStoreService::NEW_REQUEST_HASH,
            ScStoreService::ADDED_TO_TRACK_HASH,
        ];

        let mut filter = Filter::new()
            .address(addresses.to_vec())
            .event_signature(signatures.to_vec());

        // Main loop - always use RPC for real-time syncing
        loop {
            if ctx.queue.is_shutdown() {
                info!("[DAEMON_SYNC] Daemon queue is shutdown!");
                break;
            }
            
            let current_block = match self.eth.get_block_number().await {
                Ok(block) => block,
                Err(err) => {
                    error!("[DAEMON_SYNC] Error getting current block number: {}", err);
                    sleep(self.retry_timeout).await;
                    continue;
                }
            };

            let next_upper_bound = current_block.min(next_block_number.saturating_add(inclusive_block_gap));

            info!(
                "[DAEMON_SYNC] Current block: {}, Next sync block: {}, Next upper: {}",
                current_block, next_block_number, next_upper_bound
            );

            // Always use RPC in the main loop for real-time syncing
            filter = filter.from_block(next_block_number)
                .to_block(next_upper_bound);
            
            let logs = match self.eth.get_logs(&filter).await {
                Ok(logs) => logs,
                Err(err) => {
                    error!("[DAEMON_SYNC] RPC sync failed: {}", err);
                    sleep(self.retry_timeout).await;
                    continue;
                }
            };

            info!("[DAEMON_SYNC] Processing {} events from block {}", logs.len(), next_block_number);

            for ref item in &logs {
                let _ = self.handle_log(item).await;
            }

            let _ = self.batch_repo.save_batch(
                TransactionBatch {
                    from_block_number: next_block_number as i64,
                    to_block_number: next_upper_bound as i64,
                    status: TransactionStatus::Confirmed,
                }
            ).await;

            next_block_number = next_upper_bound + 1;

            if logs.is_empty() {
                info!("[DAEMON_SYNC] No new events found");
                sleep(self.empty_timeout).await;
            }
        }
    }

    async fn bulk_sync(&self) -> ClientResult<u64> {
        let next_block_number = match self.batch_repo.get_last_batch().await? {
            Some(batch) => (batch.to_block_number + 1) as u64,
            None => self.store_created_block,
        };
        
        let current_block = self.eth.get_block_number().await
            .map_err(|err| ClientError::EthError(EthError::EthRpc(err)))?;

        let initial_gap = current_block.saturating_sub(next_block_number);
        let historical_threshold = env::historical_sync_threshold();

        if initial_gap <= historical_threshold {
            info!("[DAEMON_SYNC] Gap ({}) below threshold ({}), starting with RPC sync", initial_gap, historical_threshold);
            return Ok(next_block_number);
        }

        info!("[DAEMON_SYNC] Starting initial historical sync with EthScan (gap: {})", initial_gap);

        if let Err(err) = self.sync_with_ethscan(next_block_number).await {
            error!("[DAEMON_SYNC] Initial EthScan sync failed: {}, will continue with RPC", err);
            return Err(err)?
        }

        let _ = self.batch_repo.save_batch(
            TransactionBatch {
                from_block_number: next_block_number as i64,
                to_block_number: current_block as i64,
                status: TransactionStatus::Confirmed,
            }
        ).await;

        return Ok(current_block + 1);
    }

    async fn sync_with_ethscan(&self, from_block: u64) -> Result<(), EthScanError> {
        // Sync EtherScan events
        let assetlink_address = env::assetlink_address();
        let assetlink_signatures = HashSet::from_iter(
            vec![ScAssetLinkService::SYNC_FINISH_HASH]
        );
        self.sync_contract_logs(
            assetlink_address,
            assetlink_signatures,
            from_block,
        ).await?;
        
        // Sync Open Store events
        let openstore_address = env::openstore_address();
        let openstore_signatures = HashSet::from_iter(
            vec![
                ScStoreService::BLOCK_FINALIZED_HASH,
                ScStoreService::NEW_REQUEST_HASH,
                ScStoreService::ADDED_TO_TRACK_HASH,
            ]
        );
        self.sync_contract_logs(
            openstore_address,
            openstore_signatures,
            from_block,
        ).await?;

        Ok(())
    }

    async fn sync_contract_logs(
        &self,
        address: Address,
        topics0: HashSet<B256>,
        from_block: u64,
    ) -> Result<(), EthScanError> {
        let offset = 1000;
        let checksum = address.upper_checksum();

        let mut params = GetLogsParams {
            from_block: from_block.to_string(),
            address: Some(address.to_string()),
            offset: Some(offset),

            topic0: None,
            page: None,
        };

        if topics0.len() == 1 {
            let topic0 = topics0.iter().next()
                .expect("impossible state")
                .encode_hex_with_prefix();
            params.topic0 = Some(topic0);
        }

        let mut page = 1u32;
        loop {
            params.page = Some(page);

            info!("[DAEMON_SYNC] Fetching {} logs (with topic) page {} (offset: {})", checksum, page, offset);

            let response = self.ethscan.get_logs(&params).await?;
            let results_count = response.result.len();
            info!("[DAEMON_SYNC] Got {} results for {} page {}", results_count, checksum, page);

            // Use block numbers from EthScan to fetch actual logs via RPC
            for ref entry in response.result {
                if topics0.len() == 1 {
                    self.handle_log(entry).await
                } else {
                    let Some(topic0) = entry.topic0() else {
                        continue;
                    };

                    if topics0.contains(topic0) {
                        let _ = self.handle_log(entry).await;
                    }
                }
            }

            if (results_count as u32) < offset {
                info!("[DAEMON_SYNC] Reached end of results for {} (got {} < {})", checksum, results_count, offset);
                break;
            }

            page += 1;
        }

        Ok(())
    }

    async fn handle_log(&self, item: &Log) {
        let topic0 = match item.topic0() {
            Some(topic0) => topic0.clone(),
            None => {
                warn!("Couldn't find topic0");
                return
            },
        };

        return match topic0 {
            ScAssetLinkService::SYNC_FINISH_HASH => {
                self.sync_finished.handle(item)
                    .await;
            }

            ScStoreService::BLOCK_FINALIZED_HASH => {
                self.block_finalized.handle(item)
                    .await;
            }

            ScStoreService::ADDED_TO_TRACK_HASH => {
                self.add_to_track.handle(item)
                    .await;
            }

            ScStoreService::NEW_REQUEST_HASH => {
                self.new_request.handle(item)
                    .await;
            }

            _ => ()
        }
    }
}

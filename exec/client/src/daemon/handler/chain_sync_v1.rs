use std::cmp::max;
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
use client_ethscan::client::EthScanClient;
use client_ethscan::error::EthScanError;
use client_ethscan::models::GetLogsParams;
use service_sc::assetlinks::ScAssetLinkService;
use service_sc::store::ScStoreService;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};
use client_tg::{tg_alert, tg_msg};
use net_client::node::watcher::TxWorkaround;
use service_graph::client::GraphClient;
use crate::data::repo::object_repo::ObjectRepo;
use crate::data::models::NewAsset;
use crate::data::id::{CategoryId, PlatformId};

pub struct ChainSyncHandlerV1 {
    store_created_block: u64,
    filter_block_threshold: u64,
    retry_timeout: Duration,
    empty_timeout: Duration,
    eth: Arc<Web3Provider>,
    ethscan: Arc<EthScanClient>,
	graph: Arc<GraphClient>,
	object_repo: Arc<ObjectRepo>,
    batch_repo: Arc<BatchRepo>,
    sync_finished: Arc<SyncFinishedHandler>,
    new_request: Arc<NewRequestHandler>,
    block_finalized: Arc<BlockFinalizedHandler>,
    add_to_track: Arc<AddToTrack>,
}

impl ChainSyncHandlerV1 {

    pub fn new(
        store_created_block: u64,
        filter_block_threshold: u64,
        retry_timeout: Duration,
        empty_timeout: Duration,
		eth: Arc<Web3Provider>,
		ethscan: Arc<EthScanClient>,
		graph: Arc<GraphClient>,
		object_repo: Arc<ObjectRepo>,
        batch_repo: Arc<BatchRepo>,
        sync_finished: Arc<SyncFinishedHandler>,
        new_request: Arc<NewRequestHandler>,
        block_finalized: Arc<BlockFinalizedHandler>,
        add_to_track: Arc<AddToTrack>,
    ) -> Self {
		Self { store_created_block, filter_block_threshold, retry_timeout, empty_timeout, eth, ethscan, graph, object_repo, batch_repo, sync_finished, block_finalized, new_request, add_to_track }
    }

    pub async fn handle(&self, ctx: Arc<DaemonContex>) {
        self.sync_logs_3rd_party(ctx)
            .await;
    }

    ///////////
    // RUN
    ///////////
    // TODO Delete when alchemy getLogs polls are ready for production
    async fn sync_logs_3rd_party(&self, ctx: Arc<DaemonContex>) {
        let next_block_number = match self.batch_repo.get_last_batch().await {
            Ok(block) => match block {
                Some(batch) => (batch.to_block_number + 1) as u64,
                None => self.store_created_block,
            },
            Err(e) => {
                error!("[DAEMON_SYNC] No last block found: {}", e);
                tg_alert!(format!("[DAEMON_SYNC] No last block found: {}", e));
                ctx.queue.push_sequential(DaemonAction::Shutdown)
                    .await;

                return;
            }
        };

        let offset = env::max_logs_per_request();

        // Sync EtherScan events
        let assetlink_address = env::assetlink_address();
        let mut assetlink_params = GetLogsParams {
            from_block: 0,
            to_block: None,
            address: Some(assetlink_address.checksum()),
            offset: Some(offset),

            topic0: Some(ScAssetLinkService::SYNC_FINISH_HASH.encode_hex_with_prefix()),
            page: None,
        };

        // Sync Open Store events
        let openstore_address = env::openstore_address();
        let openstore_signatures: HashSet<B256> = HashSet::from_iter(
            vec![
                ScStoreService::BLOCK_FINALIZED_HASH,
                ScStoreService::NEW_REQUEST_HASH,
                ScStoreService::ADDED_TO_TRACK_HASH,
            ]
        );
        let mut openstore_params = GetLogsParams {
            from_block: 0,
            to_block: None,
            address: Some(openstore_address.checksum()),
            offset: Some(offset),

            topic0: None,
            page: None,
        };

        let mut from_block = next_block_number;
        let mut page = 1u32;
        // Main loop - always use RPC for real-time syncing
        loop {
            if ctx.queue.is_shutdown() {
                info!("[DAEMON_SYNC] Daemon queue is shutdown!");
                break;
            }

            info!(
                "[DAEMON_SYNC] Sync logs starting block: {}",
                from_block
            );

            let last_block_number = self.eth.get_block_number().await
                .unwrap_or_else(|e| {
                    warn!("[DAEMON_SYNC] Can't sync last block number {e}");
                    from_block
                });

            assetlink_params.from_block = from_block;
            assetlink_params.to_block = Some(last_block_number);
            
            openstore_params.from_block = from_block;
            openstore_params.to_block = Some(last_block_number);

            page = 1;
            loop {
                assetlink_params.page = Some(page);

                info!("[DAEMON_SYNC] Fetching ASSETS logs (with topic) page {} (offset: {})", page, offset);
                let response = match self.ethscan.get_logs(&assetlink_params).await {
                    Ok(response) => response,
                    Err(err) => {
                        error!("[DAEMON_SYNC] Error getting ASSETS logs: {}", err);
                        tg_msg!(format!("[DAEMON_SYNC] Error getting ASSETS logs: {}", err));
                        sleep(self.retry_timeout).await;
                        continue;
                    }
                };

                let results_count = response.result.len();
                info!("[DAEMON_SYNC] Got {} results for ASSETS page {}", results_count, page);

                // Use block numbers from EthScan to fetch actual logs via RPC
                for entry in response.result.iter() {
                    self.handle_log(entry).await
                }

                if (results_count as u32) < offset {
                    info!("[DAEMON_SYNC] Reached end of results for ASSETS (got {} < {})", results_count, offset);
                    break;
                }

                page += 1;
            }

            page = 1;
            loop {
                openstore_params.page = Some(page);

                info!("[DAEMON_SYNC] Fetching OPENSTORE logs (with topic) page {} (offset: {})", page, offset);
                let response = match self.ethscan.get_logs(&openstore_params).await {
                    Ok(response) => response,
                    Err(err) => {
                        error!("[DAEMON_SYNC] Error getting OPENSTORE logs: {}", err);
                        tg_msg!(format!("[DAEMON_SYNC] Error getting OPENSTORE logs: {}", err));
                        sleep(self.retry_timeout).await;
                        continue;
                    }
                };

                let results_count = response.result.len();
                info!("[DAEMON_SYNC] Got {} results for OPENSTORE page {}", results_count, page);

                // Use block numbers from EthScan to fetch actual logs via RPC
                for entry in response.result.iter() {
                    let Some(topic0) = entry.topic0() else {
                        continue;
                    };

                    if openstore_signatures.contains(topic0) {
                        let _ = self.handle_log(entry).await;
                    }
                }

                if (results_count as u32) < offset {
                    info!("[DAEMON_SYNC] Reached end of results for OPENSTORE (got {} < {})", results_count, offset);
                    break;
                }

                page += 1;
            }

            info!("[DAEMON_SYNC] Fetching updated AppAssets since block {}", from_block);
            match self.graph.fetch_app_assets_since(from_block).await {
                Ok(apps) => {
                    match self.object_repo.update_from_graph_list(apps).await {
                        Ok(updated) => { if updated == 0 { warn!("[DAEMON_SYNC] No existing apps updated from Graph"); } },
                        Err(e) => { warn!("[DAEMON_SYNC] Graph batch update failed: {}", e); }
                    }
                }
                Err(err) => {
                    warn!("[DAEMON_SYNC] Graph fetch failed: {}", err);
                }
            }

            let _ = self.batch_repo.save_batch(
                TransactionBatch {
                    from_block_number: from_block as i64,
                    to_block_number: last_block_number as i64,
                    status: TransactionStatus::Confirmed,
                }
            ).await;

            from_block = last_block_number;

            info!("[DAEMON_SYNC] Events synced, next block: {}", from_block);
            sleep(self.empty_timeout).await;
        }
    }

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
        let checksum = address.checksum();

        let mut params = GetLogsParams {
            from_block,
            to_block: None,
            address: Some(checksum.clone()),
            offset: Some(offset),

            topic0: None,
            page: None,
        };

        if topics0.len() == 1 {
            let topic0 = topics0.iter().next()
                .expect("impossible state, no topics")
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

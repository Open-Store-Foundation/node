use crate::daemon::handler::sync::add_to_track::AddToTrack;
use crate::daemon::handler::sync::block_finalized::BlockFinalizedHandler;
use crate::daemon::data::data_sync::{DataSyncHandler, LogResultData};
use crate::daemon::handler::sync::new_req::NewRequestHandler;
use crate::daemon::handler::sync::new_req_v0::NewRequestHandlerV0;
use crate::daemon::handler::sync::sync_finish::SyncFinishedHandler;
use crate::daemon::handler::sync::sync_finish_v0::SyncFinishedHandlerV0;
use crate::daemon::launcher::{DaemonAction, DaemonContex};
use crate::data::id::{CategoryId, PlatformId};
use crate::data::models::{AssetlinkSync, NewArtifact, NewAsset, NewBuildRequest, Publishing};
use crate::data::repo::assetlink_repo::AssetlinkRepo;
use crate::data::repo::batch_repo::{BatchRepo, TransactionBatch, TransactionStatus};
use crate::data::repo::object_repo::ObjectRepo;
use crate::env;
use crate::result::{ClientError, ClientResult};
use alloy::hex::ToHexExt;
use alloy::primitives::{Address, B256};
use alloy::providers::Provider;
use alloy::rpc::types::{Filter, Log};
use client_tg::{tg_alert, tg_msg};
use codegen_contracts::ext::ToChecksum;
use net_client::node::provider::Web3Provider;
use net_client::node::result::EthError;
use net_client::node::watcher::TxWorkaround;
use service_ethscan::client::EthScanClient;
use service_ethscan::error::EthScanError;
use service_ethscan::models::GetLogsParams;
use service_graph::client::GraphClient;
use service_sc::assetlinks::ScAssetLinkService;
use service_sc::store::ScStoreService;
use std::cmp::max;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};

pub struct ChainSyncHandlerV0 {
    store_created_block: u64,
    filter_block_threshold: u64,
    retry_timeout: Duration,
    empty_timeout: Duration,
    eth: Arc<Web3Provider>,
    ethscan: Arc<EthScanClient>,
	graph: Arc<GraphClient>,
    data_sync: Arc<DataSyncHandler>,
    sync_finished: Arc<SyncFinishedHandlerV0>,
    new_request: Arc<NewRequestHandlerV0>,
}

impl ChainSyncHandlerV0 {

    pub fn new(
        store_created_block: u64,
        filter_block_threshold: u64,
        retry_timeout: Duration,
        empty_timeout: Duration,
        eth: Arc<Web3Provider>,
        ethscan: Arc<EthScanClient>,
        graph: Arc<GraphClient>,
        data_sync: Arc<DataSyncHandler>,
        sync_finished: Arc<SyncFinishedHandlerV0>,
        new_request: Arc<NewRequestHandlerV0>,
    ) -> Self {
		Self { store_created_block, filter_block_threshold, retry_timeout, empty_timeout, eth, ethscan, graph, data_sync, sync_finished, new_request }
    }

    pub async fn handle(&self, ctx: Arc<DaemonContex>) {
        self.sync_logs_3rd_party(ctx)
            .await;
    }

    ///////////
    // RUN
    ///////////
    async fn sync_logs_3rd_party(&self, ctx: Arc<DaemonContex>) {
        let next_block_number = match self.data_sync.last_sync_batch().await {
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
            address: Some(assetlink_address.lower_checksum()),
            offset: Some(offset),

            topic0: Some(ScAssetLinkService::SYNC_FINISH_HASH.encode_hex_with_prefix()),
            page: None,
        };

        // Sync Open Store events
        let openstore_address = env::openstore_address();
        let mut openstore_params = GetLogsParams {
            from_block: 0,
            to_block: None,
            address: Some(openstore_address.lower_checksum()),
            offset: Some(offset),

            topic0: Some(ScStoreService::NEW_REQUEST_HASH.encode_hex_with_prefix()),
            page: None,
        };

        let mut from_block = next_block_number;
        let mut page = 1u32;

        // TODO max size within sync
        let mut logs = Vec::with_capacity(64);
        let mut new_data = Vec::with_capacity(64);

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
                logs.extend(response.result);

                if (results_count as u32) < offset {
                    info!("[DAEMON_SYNC] Reached end of results for ASSETS (got {} < {})", results_count, offset);
                    break;
                }

                // TODO sleep for 1 sec
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
                logs.extend(response.result);

                if (results_count as u32) < offset {
                    info!("[DAEMON_SYNC] Reached end of results for OPENSTORE (got {} < {})", results_count, offset);
                    break;
                }

                // TODO sleep for 1 sec
                page += 1;
            }

            for log in logs.iter() {
                if let Some(result) = self.handle_log(log).await {
                    new_data.push(result);
                }
            }

            logs.clear();

            info!("[DAEMON_SYNC] Fetching updated AppAssets since block {}", from_block);
            let apps = match self.graph.fetch_app_assets_since(from_block).await {
                Ok(apps) => Some(apps),
                Err(err) => {
                    warn!("[DAEMON_SYNC] Graph fetch failed: {}", err);
                    None
                }
            };
            
            self.data_sync.sync(&new_data, &apps, from_block, last_block_number)
                .await;
            
            new_data.clear();

            from_block = last_block_number;

            info!("[DAEMON_SYNC] Events synced, next block: {}", from_block);
            sleep(self.empty_timeout).await;
        }
    }
    
    async fn handle_log(&self, item: &Log) -> Option<LogResultData> {
        let topic0 = match item.topic0() {
            Some(topic0) => topic0.clone(),
            None => {
                warn!("Couldn't find topic0");
                return None
            },
        };

        return match topic0 {
            ScAssetLinkService::SYNC_FINISH_HASH => {
                let result = self.sync_finished.handle(item).await;
                Some(LogResultData::FinishSync(result.0, result.1))
            }

            ScStoreService::NEW_REQUEST_HASH => {
                let result = self.new_request.handle(item).await;
                Some(LogResultData::NewRequest(result.0, result.1, result.2, None))
            }

            _ => None,
        }
    }
}

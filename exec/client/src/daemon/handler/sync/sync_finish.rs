use crate::daemon::data::object_factory::ObjectFactory;
use crate::data::models::AssetlinkSync;
use crate::data::repo::assetlink_repo::AssetlinkRepo;
use crate::data::repo::error_repo::ErrorRepo;
use crate::data::repo::object_repo::ObjectRepo;
use alloy::hex::ToHexExt;
use alloy::primitives::{Address, Bytes, LogData, TxHash, U256};
use alloy::rpc::types::Log;
use core_std::trier::SyncTrier;
use service_sc::assetlinks::{AssetlinkStatusCode, ScAssetLinkService};
use service_sc::obj::ScObjService;
use service_sc::store::ScStoreService;
use std::sync::Arc;
use tracing::{error, info};
use codegen_contracts::ext::ToChecksum;

pub struct SyncFinishedHandler {
    factory: Arc<ObjectFactory>,
    app_provider: Arc<ScObjService>,
    obj_repo: Arc<ObjectRepo>,
    assetlink_repo: Arc<AssetlinkRepo>,
    error_repo: Arc<ErrorRepo>,
}

impl SyncFinishedHandler {
    
    pub fn new(
        factory: Arc<ObjectFactory>,
        app_provider: Arc<ScObjService>,
        obj_repo: Arc<ObjectRepo>,
        assetlink_repo: Arc<AssetlinkRepo>,
        error_repo: Arc<ErrorRepo>,
    ) -> Self {
        Self {
            factory,
            app_provider,
            obj_repo,
            assetlink_repo,
            error_repo,
        }
    }

    pub async fn handle(&self, item: &Log) {
        let result = ScAssetLinkService::decode_finalize_log(item.as_ref());
        let (obj_address, status, owner_version) = match result {
            Ok(log) => (log.data.app, log.data.status, log.data.version),
            Err(e) => {
                if let Some(tx_hash) = item.transaction_hash {
                    let _ = self.error_repo.insert_fatal_tx(tx_hash.encode_hex_with_prefix())
                        .await;
                }

                error!("[SYNC_FINISH_HANDLER] Failed to decode finalize log: {}", e);
                return;
            }
        };

        self.handle_internal(item.transaction_hash, obj_address, status.to(), owner_version.to()).await;
    }

    async fn handle_internal(
        &self,
        transaction_hash: Option<TxHash>,
        obj_address: Address,
        status: i32,
        owner_version: i64,
    ) {
        let object_addr = obj_address.checksum();
        let website = match self.app_provider.website(obj_address, owner_version).await {
            Ok(website) => website,
            Err(e) => {
                error!("[SYNC_FINISH_HANDLER] Failed to get website: {}", e);
                if let Some(tx_hash) = transaction_hash {
                    let _ = self.error_repo.insert_error_tx(tx_hash.encode_hex_with_prefix())
                        .await;
                }

                return;
            }
        };

        let has_obj = self.obj_repo.has_by_address(object_addr.as_ref()).await;
        if !has_obj {
            if let Ok(obj) = self.factory.create_obj(obj_address).await {
                let _ = self.obj_repo.insert_or_update(&obj)
                    .await;
            } else {
                error!("[SYNC_FINISH_HANDLER] Failed to sync obj - {}!", object_addr);
            };
        }

        let verification = AssetlinkSync {
            asset_address: object_addr,
            owner_version,
            domain: website,
            status,
        };

        let mut sync = SyncTrier::new(1_000, 1.0, 2);
        while sync.iterate().await {
            let result = self.assetlink_repo
                .insert_assetlink_status(&verification)
                .await;

            if let Err(e) = result {
                error!("[SYNC_FINISH_HANDLER] Failed to insert assetlink status: {}", e);
            } else {
                break;
            }
        }

        if sync.is_failed() {
            if let Some(tx_hash) = transaction_hash {
                error!("[SYNC_FINISH_HANDLER] Failed to handle finish sync event!");
                let _ = self.error_repo.insert_error_tx(tx_hash.encode_hex_with_prefix())
                    .await;
            }
        }
    }
}

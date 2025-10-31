use crate::daemon::data::object_factory::ObjectFactory;
use crate::data::id::TrackId;
use crate::data::repo::error_repo::ErrorRepo;
use crate::data::repo::object_repo::ObjectRepo;
use crate::data::repo::publishing_repo::PublishingRepo;
use alloy::hex::ToHexExt;
use alloy::primitives::{Address, B256};
use alloy::rpc::types::Log;
use core_std::trier::SyncTrier;
use service_sc::store::ScStoreService;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;
use tracing::{error, info};
use codegen_contracts::ext::ToChecksum;

pub struct AddToTrack {
    factory: Arc<ObjectFactory>,
    obj_repo: Arc<ObjectRepo>,
    publishing_repo: Arc<PublishingRepo>,
    error_repo: Arc<ErrorRepo>,
}

impl AddToTrack {

    pub fn new(
        factory: Arc<ObjectFactory>,
        obj_repo: Arc<ObjectRepo>,
        publishing_repo: Arc<PublishingRepo>,
        error_repo: Arc<ErrorRepo>,
    ) -> Self {
        Self {
            factory,
            obj_repo,
            publishing_repo,
            error_repo,
        }
    }
    
    pub async fn handle(&self, item: &Log) {
        let log = ScStoreService::decode_add_to_track(item.as_ref());

        let (target, track_id, version_code) = match log {
            Ok(log) => (log.data.target, log.data.trackId, log.data.versionCode),
            Err(e) => {
                if let Some(tx_hash) = item.transaction_hash {
                    let _ = self.error_repo.insert_fatal_tx(tx_hash.encode_hex_with_prefix())
                        .await;
                }

                error!("[ADD_TO_TRACK] Failed to decode log: {}]", e);
                return;
            }
        };

        self.handle_internal(
            item.transaction_hash,
            target,
            TrackId::from(track_id.to::<i32>()),
            version_code.to(),
        ).await;
    }

    async fn handle_internal(
        &self,
        transaction_hash: Option<B256>,
        target: Address,
        track_id: TrackId,
        version_code: i64,
    ) {
        info!("[ADD_TO_TRACK] Handling log: target - {} | track - {} | version - {}!", target.checksum(), track_id, version_code);

        let mut sync = SyncTrier::new(1, 1.0, 2);
        while sync.iterate().await {
            let publish = self.factory.create_publishing(
                target,
                track_id.clone(),
                version_code,
            );

            if let Err(e) = self.publishing_repo.insert_or_update(&publish).await {
                error!("[ADD_TO_TRACK] Failed to insert publishing: {}", e)
            } else {
                break
            };
        }

        if sync.is_failed() {
            if let Some(tx_hash) = transaction_hash {
                error!("[ADD_TO_TRACK] Failed to insert publishing, saving tx_hash...");
                let _ = self.error_repo.insert_error_tx(tx_hash.encode_hex_with_prefix())
                    .await;
            }
        }
        
        info!("[ADD_TO_TRACK] Handling log SUCCESSFULLY!");
    }
}

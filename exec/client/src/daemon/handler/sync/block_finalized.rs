use std::str::FromStr;
use crate::daemon::data::object_factory::ObjectFactory;
use crate::data::id::TrackId;
use crate::data::models::{BuildRequest, NewAsset, NewBuildRequest, Publishing};
use crate::data::repo::artifact_repo::ArtifactRepo;
use crate::data::repo::error_repo::ErrorRepo;
use crate::data::repo::object_repo::ObjectRepo;
use crate::data::repo::publishing_repo::PublishingRepo;
use crate::data::repo::validation_repo::ValidationRepo;
use alloy::hex::ToHexExt;
use alloy::primitives::{Address, TxHash};
use alloy::rpc::types::Log;
use codegen_block::block::ValidationBlock;
use core_std::trier::SyncTrier;
use prost::Message;
use service_sc::store::ScStoreService;
use std::sync::Arc;
use tracing::{error, warn, info};
use codegen_contracts::ext::ToChecksum;

pub struct BlockFinalizedHandler {
    factory: Arc<ObjectFactory>,
    store_provider: Arc<ScStoreService>,
    obj_repo: Arc<ObjectRepo>,
    publishing_repo: Arc<PublishingRepo>,
    art_repo: Arc<ArtifactRepo>,
    validation_repo: Arc<ValidationRepo>,
    error_repo: Arc<ErrorRepo>,
}

impl BlockFinalizedHandler {
    pub fn new(
        factory: Arc<ObjectFactory>,
        store_provider: Arc<ScStoreService>,
        obj_repo: Arc<ObjectRepo>,
        publishing_repo: Arc<PublishingRepo>,
        art_repo: Arc<ArtifactRepo>,
        validation_repo: Arc<ValidationRepo>,
        error_repo: Arc<ErrorRepo>,
    ) -> Self {
        Self {
            factory,
            store_provider,
            publishing_repo,
            obj_repo,
            art_repo,
            validation_repo,
            error_repo,
        }
    }

    pub async fn handle(&self, item: &Log) {
        let result = ScStoreService::decode_block_finalize(item.as_ref());
        let (_, _, object_id) = match result {
            Ok(result) => (
                result.data.blockId,
                result.data.creator,
                result.data.objectId,
            ),
            Err(err) => {
                if let Some(tx_hash) = item.transaction_hash {
                    let _ = self
                        .error_repo
                        .insert_fatal_tx(tx_hash.encode_hex_with_prefix())
                        .await;
                }

                error!("[BLOCK_FINALIZED] Failed to decode block finalize: {}", err);
                return;
            }
        };

        let Ok(artifact_id) = TxHash::try_from(object_id.as_ref()) else {
            if let Some(tx_hash) = item.transaction_hash {
                let _ = self
                    .error_repo
                    .insert_fatal_tx(tx_hash.encode_hex_with_prefix())
                    .await;
            }

            error!("[BLOCK_FINALIZED] Failed to decode object id to TxHash.");
            return;
        };

        self.handle_internal(item.transaction_hash, item.block_timestamp, artifact_id).await;
    }

    async fn handle_internal(&self, transaction_hash: Option<TxHash>, translation_time: Option<u64>, artifact_id: TxHash) {
        let block_data = match self.store_provider.get_block_data(artifact_id).await {
            Ok(data) => match data {
                Some(data) => data,
                None => {
                    error!("[BLOCK_FINALIZED] Block data not found");
                    if let Some(tx_hash) = transaction_hash {
                        let _ = self
                            .error_repo
                            .insert_fatal_tx(tx_hash.encode_hex_with_prefix())
                            .await;
                    }

                    return;
                }
            },
            Err(err) => {
                error!("[BLOCK_FINALIZED] Failed to get block data: {}", err);
                if let Some(tx_hash) = transaction_hash {
                    let _ = self
                        .error_repo
                        .insert_error_tx(tx_hash.encode_hex_with_prefix())
                        .await;
                }

                return;
            }
        };

        let block = match ValidationBlock::decode(block_data.as_ref()) {
            Ok(block) => block,
            Err(err) => {
                error!("[BLOCK_FINALIZED] Failed to decode block: {}", err);
                if let Some(tx_hash) = transaction_hash {
                    let _ = self
                        .error_repo
                        .insert_fatal_tx(tx_hash.encode_hex_with_prefix())
                        .await;
                }

                return;
            }
        };

        if block.requests.len() == 0 {
            return;
        }

        let mut requests: Vec<NewBuildRequest> = Vec::with_capacity(block.requests.len());
        let mut results: Vec<String> = vec![];
        let mut artifacts: Vec<(String, i64)> = vec![];
        let mut publishings: Vec<Publishing> = vec![];

        for result in block.requests {
            let version = result.object_version;
            let owner_version = result.owner_version;

            let obj_address_str = result.asset_address.checksum();
            let obj_address = Address::from_str(obj_address_str.as_str());
            let obj_address = match obj_address {
                Ok(obj_address) => obj_address,
                Err(err) => {
                    error!("[BLOCK_FINALIZED] Failed to parse object address: {}", err);
                    continue;
                }
            };

            let build_request = self.factory.create_build_request(
                result.request_id,
                obj_address,
                result.track_id as u8,
                Some(result.status as i32),
                version,
                owner_version,
                translation_time,
            );

            requests.push(build_request);

            if version > 0 {
                results.push(obj_address_str.clone());
            }

            if result.artifact_protocol > 0 {
                artifacts.push((obj_address_str, version));
            }

            if result.track_id > 0 {
                let publishing = self.factory.create_publishing(
                    obj_address,
                    TrackId::from(result.track_id as i32),
                    version,
                );

                publishings.push(publishing);
            }
        }

        let missing_obj = self.obj_repo.find_obj_missing_addresses(results).await;
        let mut obj_to_insert = Vec::with_capacity(missing_obj.len());
        for address in missing_obj {
            let Ok(address) = Address::from_str(address.as_ref()) else {
                warn!("[BLOCK_FINALIZED] Can't decode address");
                continue
            };
            
            let obj = match self.factory.create_obj(address).await {
                Ok(obj) => obj,
                Err(e) => {
                    error!("[BLOCK_FINALIZED] Can't fetch object data: {}, {}", address, e);
                    continue;
                }
            };

            obj_to_insert.push(obj);
        }

        let missing_artifacts = self.art_repo.find_artifact_missing_refs(artifacts).await;
        let mut art_to_insert = Vec::with_capacity(missing_artifacts.len());
        for (address, version) in missing_artifacts {
            let Ok(address) = Address::from_str(address.as_ref()) else {
                warn!("[BLOCK_FINALIZED] Can't decode address");
                continue
            };

            let art = match self.factory.create_artifact(address, version).await {
                Ok(art) => art,
                Err(e) => {
                    error!("[BLOCK_FINALIZED] Can't fetch artifact data: {}, {}", address, e);
                    continue;
                }
            };

            art_to_insert.push(art);
        }

        let mut sync = SyncTrier::new(1, 1.0, 3);

        'retrier: while sync.iterate().await {
            if let Ok(transaction) = self.obj_repo.start().await {
                info!("[BLOCK_FINALIZED] Starting transaction!");
                
                for request in &requests {
                    let res = self.validation_repo.insert_or_update(request).await;

                    if let Err(err) = res {
                        error!("[BLOCK_FINALIZED] Failed to insert request: {}", err);
                        let _ = transaction.rollback().await;
                        continue 'retrier
                    }
                }

                for publish in &publishings {
                    let res = self.publishing_repo.insert_or_update(publish).await;

                    if let Err(err) = res {
                        error!("[BLOCK_FINALIZED] Failed to insert publishing: {}", err);
                        let _ = transaction.rollback().await;
                        continue 'retrier
                    }
                }

                for obj in &obj_to_insert {
                    let res = self.obj_repo.insert_or_update(obj).await;

                    if let Err(err) = res {
                        error!("[BLOCK_FINALIZED] Failed to insert object: {}", err);
                        let _ = transaction.rollback().await;
                        continue 'retrier
                    }
                }

                for art in &art_to_insert {
                    let res = self.art_repo.insert_artifact(art).await;

                    if let Err(err) = res {
                        error!("[BLOCK_FINALIZED] Failed to insert artifact: {}", err);
                        let _ = transaction.rollback().await;
                        continue 'retrier
                    }
                }

                if let Err(e) = transaction.commit().await {
                    error!("[BLOCK_FINALIZED] Can't commit transaction {}", e);
                    continue 'retrier
                } else {
                    info!("[BLOCK_FINALIZED] Transaction committed!");
                    break 'retrier
                };
            }
        }

        if sync.is_failed() {
            if let Some(tx_hash) = transaction_hash {
                error!("[BLOCK_FINALIZED] Failed sync data, saving tx_hash...");
                let _ = self.error_repo.insert_error_tx(tx_hash.encode_hex_with_prefix())
                    .await;
            }
        }
    }
}

#[tokio::test]
async fn check_decode() {
    let data = "08013AD60108011001180122423078303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303139353335382A2A3078303333323962623044333644323345416344623132354335333030414644393538303845396261393001380140014A06626C616B65335240463335434342364534374238393338394633383041324239423330424133343445373645383433313831463132354632333437364536384646423732434341435A0C08FC9A0310FCBA0318D1CA036001";
    
    let result = ValidationBlock::decode(hex::decode(data).unwrap().as_slice());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().id, 1);
}
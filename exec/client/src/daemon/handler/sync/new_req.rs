use crate::daemon::data::obj_info_provider::DaemonFactory;
use crate::data::id::{ObjTypeId, ReqTypeId};
use crate::data::models::NewAsset;
use crate::data::repo::artifact_repo::ArtifactRepo;
use crate::data::repo::error_repo::ErrorRepo;
use crate::data::repo::object_repo::ObjectRepo;
use crate::data::repo::publishing_repo::PublishingRepo;
use crate::data::repo::validation_repo::ValidationRepo;
use alloy::hex::ToHexExt;
use alloy::primitives::ruint::aliases::U256;
use alloy::primitives::Address;
use alloy::rpc::types::Log;
use alloy::sol_types::{SolEvent, SolType};
use codegen_block::status::ApkValidationStatus;
use codegen_contracts::contracts::OpenStore::NewRequest;
use codegen_contracts::ext::ToChecksum;
use service_sc::obj::ScObjService;
use service_sc::store::{AndroidObjRequestData, ScStoreService};
use std::str::FromStr;
use std::sync::Arc;
use std::u64;
use tracing::{error, info, warn};

pub struct NewRequestHandler {
    factory: Arc<DaemonFactory>,
    obj_repo: Arc<ObjectRepo>,
    art_repo: Arc<ArtifactRepo>,
    validation_repo: Arc<ValidationRepo>,
    error_repo: Arc<ErrorRepo>,
}

impl NewRequestHandler {
    
    pub fn new(
        factory: Arc<DaemonFactory>,
        obj_repo: Arc<ObjectRepo>,
        art_repo: Arc<ArtifactRepo>,
        validation_repo: Arc<ValidationRepo>,
        error_repo: Arc<ErrorRepo>,
    ) -> Self {
        Self {
            factory,
            obj_repo,
            art_repo,
            validation_repo,
            error_repo,
        }
    }

    pub async fn handle(&self, item: &Log) {
        let result = ScStoreService::decode_new_request(item.as_ref());

        let (request_type, obj, request_id, data) = match result {
            Ok(result) => (
                result.data.reqType,
                result.data.target,
                result.data.requestId.to::<u64>(),
                result.data.data,
            ),
            Err(e) => {
                if let Some(tx_hash) = item.transaction_hash {
                    let _ = self.error_repo.insert_fatal_tx(tx_hash.encode_hex_upper())
                        .await;
                }

                error!("[NEW_REQ_HANDLER] Can't decode app's event data: {}", e);
                return;
            }
        };

        self.handle_internal(request_id, request_type.to(), obj, data.as_ref()).await;
    }

    async fn handle_internal(
        &self,
        request_id: u64,
        request_type: u8,
        obj: Address,
        data: &[u8],
    ) {
        info!("[NEW_REQ_HANDLER] Start handling...");
        let address = obj.upper_checksum();
        info!("[NEW_REQ_HANDLER] Request type: {}, obj: {}, request id: {}", request_type, address, request_id);

        let mut res_request = None;
        let mut res_artifact = None;
        let mut res_object = None;

        let request_type_id = ReqTypeId::from(request_type);

        match request_type_id {
            ReqTypeId::AndroidBuild => {
                let result = AndroidObjRequestData::abi_decode_sequence(data.as_ref());
                let (version, owner_version, track_id) = match result {
                    Ok(result) => result,
                    Err(e) => {
                        error!("[NEW_REQ_HANDLER] Can't decode AndroidObjRequestData {}", e);
                        return;
                    }
                };

                let artifact = self.factory.create_artifact(obj, version).await;

                match artifact {
                    Ok(artifact) => {
                        res_artifact = Some(artifact);
                    }
                    Err(e) => {
                        error!("[NEW_REQ_HANDLER] Can't create artifact {}", e);
                    }
                }

                let build = self.factory
                    .create_build_request(request_id, obj, track_id, None, version, owner_version);

                res_request = Some(build);
            }
            _ => {
                warn!("[NEW_REQ_HANDLER] Unknown request type: {}", request_type)
            }
        }

        let has_obj = self.obj_repo.has_by_address(address.as_ref()).await;
        if !has_obj {
            match self.factory.create_obj(obj).await {
                Ok(obj) => {
                    res_object = Some(obj);
                }
                Err(e) => {
                    error!("[NEW_REQ_HANDLER] Can't create object {}", e);
                }
            }
        }
        
        if let Ok(transaction) = self.obj_repo.start().await {
            if let Some(request) = res_request {
                if let Err(e) = self.validation_repo.insert_or_update(&request).await {
                    error!("[NEW_REQ_HANDLER] Can't insert build {}", e);
                };
            }

            if let Some(artifact) = res_artifact {
                if let Err(e) = self.art_repo.insert_artifact(&artifact).await {
                    error!("[NEW_REQ_HANDLER] Can't insert artifact {}", e);
                };
            }

            if let Some(obj) = res_object {
                if let Err(e) = self.obj_repo.insert_or_update(&obj).await {
                    error!("[NEW_REQ_HANDLER] Can't insert object {}", e);
                };
            }

            if let Err(e) = transaction.commit().await {
                error!("[NEW_REQ_HANDLER] Can't commit transaction {}", e);
            };
        }
    }
}

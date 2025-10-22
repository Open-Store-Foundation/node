use crate::daemon::data::object_factory::ObjectFactory;
use crate::data::models::{AssetlinkSync, NewAsset, ValidationProof};
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
use crate::util::proof_verifier::ProofVerifier;

pub struct SyncFinishedHandlerV0 {
    factory: Arc<ObjectFactory>,
    verifier: Arc<ProofVerifier>,
    app_provider: Arc<ScObjService>,
    obj_repo: Arc<ObjectRepo>,
    assetlink_repo: Arc<AssetlinkRepo>,
    error_repo: Arc<ErrorRepo>,
}

impl SyncFinishedHandlerV0 {
    
    pub fn new(
        factory: Arc<ObjectFactory>,
        verifier: Arc<ProofVerifier>,
        app_provider: Arc<ScObjService>,
        obj_repo: Arc<ObjectRepo>,
        assetlink_repo: Arc<AssetlinkRepo>,
        error_repo: Arc<ErrorRepo>,
    ) -> Self {
        Self {
            factory,
            verifier,
            app_provider,
            obj_repo,
            assetlink_repo,
            error_repo,
        }
    }

    pub async fn handle(&self, item: &Log) -> (Option<AssetlinkSync>, Option<ValidationProof>) {
        let result = ScAssetLinkService::decode_finalize_log(item.as_ref());
        
        let (obj_address, status, owner_version) = match result {
            Ok(log) => (log.data.app, log.data.status, log.data.version),
            Err(e) => {
                if let Some(tx_hash) = item.transaction_hash {
                    let _ = self.error_repo.insert_fatal_tx(tx_hash.encode_hex_with_prefix())
                        .await;
                }

                error!("[SYNC_FINISH_HANDLER] Failed to decode finalize log: {}", e);
                return (None, None);
            }
        };

       return self.handle_internal(item.transaction_hash, obj_address, status.to(), owner_version.to()).await;
    }

    async fn handle_internal(
        &self,
        transaction_hash: Option<TxHash>,
        obj_address: Address,
        status: u32,
        owner_version: u64,
    ) -> (Option<AssetlinkSync>, Option<ValidationProof>) {
        let object_addr = obj_address.lower_checksum();
        let website = match self.app_provider.website(obj_address, owner_version).await {
            Ok(website) => website,
            Err(e) => {
                error!("[SYNC_FINISH_HANDLER] Failed to get website: for asset {} with version {}, error: {}", object_addr, owner_version, e);
                if let Some(tx_hash) = transaction_hash {
                    let _ = self.error_repo.insert_error_tx(tx_hash.encode_hex_with_prefix())
                        .await;
                }

                return (None, None);
            }
        };
        
        let proofs_res = self.app_provider.get_owner_proof_v0(obj_address, owner_version).await;
        let owner_proofs = if let Ok(proofs) = proofs_res {
            if let Some(proofs) = proofs { // TODO combine let
                proofs 
            } else { 
                error!("[SYNC_FINISH_HANDLER] Failed to get proofs for asset {} with version {}", object_addr, owner_version);
                return (None, None);      
            }
        } else {
            error!("[SYNC_FINISH_HANDLER] Failed to get proofs for asset {} with version {}", object_addr, owner_version);
            return (None, None);       
        };

        let result = self.verifier.verify_ownership_proofs_raw(
            obj_address, &owner_proofs.data.fingerprints, &owner_proofs.proofs, &owner_proofs.certs
        );

        let proof = ValidationProof {
            object_address: object_addr.clone(),
            owner_version,
            status: result.code(),
        };
        
        let verification = AssetlinkSync {
            object_address: object_addr,
            domain: website,
            owner_version,
            status,
        };
        
        return (Some(verification), Some(proof));
    }
}

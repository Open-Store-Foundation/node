use crate::android::build::AndroidBuildVerifier;
use codegen_block::status::{ApkResult, ApkValidationStatus};
use crate::android::verifier::ApkBuildVerifier;
use crate::data::file_storage::FileStorage;
use crate::data::validation_repo::ValidationRepo;
use crate::result::{ValidatorError, ValidatorResult};
use alloy::dyn_abi::SolType;
use alloy::primitives::{Address, Bytes};
use cloud_gf::client::GreenfieldClient;
use codegen_block::block::{ValidationProofs, ValidationResult};
use codegen_block::FileHashAlgo;
use core_std::trier::SyncTrier;
use core_std::{arc, finger, hexer};
use dashmap::{DashMap, DashSet};
use openssl::hash::MessageDigest;
use openssl::sha::sha256;
use openssl::sign::Verifier;
use openssl::x509::X509;
use prost::Message;
use service_sc::assetlinks::ScAssetLinkService;
use service_sc::obj::ScObjService;
use service_sc::store::AndroidObjRequestData;
use std::collections::{HashMap, HashSet};
use std::io;
use std::io::{ErrorKind, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use alloy::hex::ToHexExt;
use alloy::signers::k256::elliptic_curve::pkcs8::der::Writer;
use blake3::Hash;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::sync::{Mutex, MutexGuard};
use tokio::time::sleep;
use tracing::{error, info};
use codegen_contracts::ext::ToChecksum;
use crate::utils::blake::blake3;

#[derive(Clone)]
pub struct AndroidValidator {
    lockers: Arc<DashSet<u64>>,
    locker: Arc<Mutex<()>>,
    greenfield: Arc<GreenfieldClient>,
    verifier: Arc<AndroidBuildVerifier>,
    file_storage: Arc<FileStorage>,
    obj_service: Arc<ScObjService>,
    validation_repo: Arc<ValidationRepo>,
}

impl AndroidValidator {

    pub fn new(
        greenfield: &Arc<GreenfieldClient>,
        verifier: &Arc<AndroidBuildVerifier>,
        file_storage: &Arc<FileStorage>,
        obj_service: Arc<ScObjService>,
        validation_repo: Arc<ValidationRepo>,
    ) -> Self {
        Self {
            lockers: arc!(DashSet::with_capacity(128)),
            locker: arc!(Mutex::new(())),
            
            greenfield: greenfield.clone(),
            verifier: verifier.clone(),
            file_storage: file_storage.clone(),
            obj_service,
            validation_repo
        }
    }
}

impl AndroidValidator {

    // TODO v2 extract to another obj
    pub async fn validate_request(
        &self,
        request_type: u8,
        target: Address,
        request_id: u64,
        data: &[u8],
    ) -> ValidationResult {
        if let Some(result) = self.validation_repo.get_result(request_id).await {
            return result;
        }
        
        loop {
            if self.lockers.contains(&request_id) {
                sleep(Duration::from_secs(5)).await;
                continue
            }
            
            let guard = self.locker.lock().await;
            
            // Double check
            if self.lockers.contains(&request_id) {
                drop(guard);
                continue
            }

            self.lockers.insert(request_id);
            
            drop(guard);
            
            break
        }

        // Double check
        if let Some(result) = self.validation_repo.get_result(request_id).await {
            self.lockers.remove(&request_id);
            return result;
        }

        let response = self.validate_request_internal(request_type, target, request_id, data)
            .await;

        let _ = self.file_storage.erase_request(request_id)
            .await;

        self.validation_repo.save_request(request_id, &response)
            .await;

        self.lockers.remove(&request_id);

        return response;
    }

    async fn validate_request_internal(
        &self,
        request_type: u8,
        target: Address,
        request_id: u64,
        data: &[u8],
    ) -> ValidationResult {
        let mut request = ValidationResult::default_with(
            request_id, request_type, target.lower_checksum()
        );

        let result = AndroidObjRequestData::abi_decode_sequence(data.as_ref());
        let (version_code, owner_version, track_id) = match result {
            Ok(result) => result,
            Err(e) => {
                error!("[VALIDATE_COMMON] Can't decode AndroidObjRequestData {}", e);
                request.status = ApkValidationStatus::Unavailable.code();
                return request;
            }
        };

        request.track_id = track_id as u32;
        request.object_version = version_code;
        request.owner_version = owner_version;

        let result = self.obj_service.get_artifact(target, version_code)
            .await;

        let (artifact_ref, artifact_protocol, checksum) = match result {
            Ok(result) => (hexer::encode_lower_pref(&result.referenceId), result.protocolId as i32, result.checksum),
            Err(e) => {
                error!("[VALIDATE_COMMON] Can't get artifact info for {version_code}: {}!", e);
                request.status = ApkValidationStatus::Unavailable.code();
                return request;
            }
        };

        request.artifact_protocol = artifact_protocol;
        request.artifact_ref_id = artifact_ref.clone();

        let result = self.load_request_data(request_id, &artifact_ref)
            .await;

        let (apk_path, blake_hash) = match result {
            Ok(result) => result,
            Err(e) => {
                error!("[VALIDATE_COMMON] Can't load artifact content for {version_code}: {}!", e);
                request.status = ApkValidationStatus::HashMismatch.code();
                return request;
            }
        };
        
        if !checksum.eq(blake_hash.as_bytes()) {
            error!(
                "[VALIDATE_COMMON] Checksum mismatch {} and {}",
                checksum.encode_hex_with_prefix(),
                blake_hash.to_string()
            );
            request.status = ApkValidationStatus::HashMismatch.code();
            return request;
        }
        
        let result = self.verifier.verify(&apk_path)
            .await;

        if let Some(ref meta) = result.offsets {
            request.proofs = Some(
                ValidationProofs {
                    eocd_from: meta.eocd,
                    cd_from: meta.cd,
                    sb_from: meta.sign,
                }
            );
        }

        request.file_hash_algorithm = FileHashAlgo::Blake3.code();
        request.file_hash = hexer::encode_lower_pref(checksum);

        if result.status != ApkValidationStatus::Success {
            request.status = result.status.code();
            return request;
        }

        // Statuses
        let certs = match result.data {
            Some(ref data) => data.certs.clone(),
            None => {
                request.status = ApkValidationStatus::NoCertificatesFound.code();
                return request;
            }
        };

        let result = self.verify_ownership_proofs(target, owner_version, certs)
            .await;

        if let Err(status) = result {
            request.status = status.code();
            return request;
        }

        request.status = ApkValidationStatus::Success.code();
        return request
    }

    // TODO v2 check version from manifest
    // TODO v2 batch download
    // TODO v2 check content-length
    // TODO v2 check protocol
    async fn load_request_data(&self, request_id: u64, object_id: &String) -> ValidatorResult<(PathBuf, Hash)> {
        let mut trier = SyncTrier::new(5, 1.0, 3);
        let mut hasher = blake3::Hasher::new();

        'trier: while trier.iterate().await {
            let mut path = match self.file_storage.prepare_request(request_id).await  {
                Ok(path) => path,
                Err(e) => {
                    error!("[VALIDATE_COMMON] Can't prepare file! {}", e);
                    continue
                }
            };


            let mut file = match self.file_storage.file_write(&path).await  {
                Ok(file) => file,
                Err(e) => {
                    error!("[VALIDATE_COMMON] Can't create file! {}", e);
                    continue
                }
            };

            let mut writer = BufWriter::new(&mut file);

            let mut response = self.greenfield.get_object(object_id.as_str())
                .await?;
            
            let mut downloaded_bytes = 0;
            loop {
                let chunk_result = response.chunk().await;

                match chunk_result {
                    Ok(chunk) => match chunk {
                        Some(chunk) => {
                            if let Err(e) = hasher.write_all(&chunk) {
                                error!("[VALIDATE_COMMON] Error writing chunk to file: {}", e);
                                continue 'trier
                            }
                            
                            if let Err(e) = writer.write_all(&chunk).await {
                                error!("[VALIDATE_COMMON] Error writing chunk to file: {}", e);
                                continue 'trier
                            }

                            downloaded_bytes += chunk.len();
                        }
                        None => {
                            info!("[VALIDATE_COMMON] Download finished");
                            let _  = writer.flush().await;
                            break 'trier
                        }
                    }
                    Err(e) => {
                        error!("[VALIDATE_COMMON] Error getting chunk from stream: {}", e);
                        continue 'trier
                    }
                }
            }
        }

        if trier.is_failed() {
            error!("[VALIDATE_COMMON] Too many retries!");
            let _ = self.file_storage.erase_request(request_id)
                .await;

            return Err(
                ValidatorError::Io(
                    io::Error::from(ErrorKind::Interrupted)
                )
            )
        }
        
        let file_hash = hasher.finalize();

        let local_path = self.file_storage.finalize_request(request_id)
            .await?;

        return Ok((local_path, file_hash));
    }

    // TODO v2 to separated object
    async fn verify_ownership_proofs(
        &self,
        target: Address,
        owner_version: u64,
        certs: Vec<X509>,
    ) -> ApkResult<()> {
        let remote_state = self.obj_service.get_owner_data(target, owner_version)
            .await
            .map_err(|err| ApkValidationStatus::IncorrectEncryptionData)?;

        let proofs: HashMap<String, Bytes> = remote_state.fingerprints.iter().
            zip(remote_state.proofs)
            .map(|(finger, proof)| {
                (hex::encode_upper(finger.as_slice()), proof)
            })
            .collect();

        let mut cert_to_hash = HashMap::<String, X509>::new();

        for item in certs {
            let der = item.to_der();

            let Ok(der) = der else {
                return Err(ApkValidationStatus::IncorrectCertFormat);
            };

            let hash = hex::encode_upper(sha256(der.as_slice()));
            cert_to_hash.insert(hash, item);
        }

        let owner_address = target.lower_checksum();
        for (ref pub_key_hash, ref cert) in cert_to_hash {
            let proof = proofs.get(pub_key_hash)
                .ok_or_else(|| ApkValidationStatus::ProofNotFound)?;

            let pub_key = cert.public_key()
                .map_err(|_| ApkValidationStatus::IncorrectEncryptionData)?;

            let mut verifier = Verifier::new(MessageDigest::sha256(), &pub_key)
                .map_err(|_| ApkValidationStatus::IncorrectEncryptionData)?;

            let data = format!("{}::{}", owner_address, finger::sha256(pub_key_hash));
            let is_valid = verifier.verify_oneshot(proof, data.as_bytes())
                .map_err(|_| ApkValidationStatus::IncorrectEncryptionData)?;

            if !is_valid {
                return Err(ApkValidationStatus::InvalidProof)
            }
        }

        return Ok(())
    }
}

use std::collections::HashMap;
use std::path::Path;

use bytes::Bytes;
use openssl::hash::MessageDigest;
use openssl::pkey::PKey;
use openssl::sign::{RsaPssSaltlen, Verifier};
use openssl::x509::X509;
use tokio::fs::File;
use tracing::error;
use core_std::endian::get_u32_le;
use crate::android::apk::chunker::ApkChunker;
use crate::android::apk::crypto::{ApkChunkDigest, ApkSignature};
use crate::android::apk::parser::{ApkBlockInfo, ApkOffsets, ApkParser};
use crate::android::apk::reader::{get_len_pref_copy, get_len_pref_slice};
use crate::android::status::{ApkValidationStatus, ApkResult};
use crate::utils::blake;

const APK_V2_BLOCK_ID: u32 = 0x7109871a;
const MAX_V2_SIGNERS: usize = 10;
const STRIPPING_PROTECTION_ATTR_ID: u32 = 0xbeeff00d;
const SF_ATTRIBUTE_ANDROID_APK_SIGNED_ID: u32 = 2;
const SF_ATTRIBUTE_ANDROID_APK_SIGNED_V3_ID: u32 = 3;

#[derive(Default, Debug)]
pub struct ApkVerificationResult {
    pub status: ApkValidationStatus,
    pub offsets: Option<ApkOffsets>,
    pub data: Option<ApkVerificationData>,
}

#[derive(Debug)]
pub struct ApkVerificationData {
    pub certs: Vec<X509>,
    pub digest: HashMap<ApkChunkDigest, Bytes>,
}

pub struct ApkVerifierV2 {
    parser: ApkParser,
    chunker: ApkChunker,
}

impl ApkVerifierV2 {

    pub fn new(parser: ApkParser, chunk: ApkChunker) -> Self {
        Self { parser, chunker: chunk }
    }

    pub async fn verify(
        &self,
        apk_file: impl AsRef<Path>,
        verify_integrity: bool,
    ) -> ApkVerificationResult {
        let mut result = ApkVerificationResult::default();
        
        // Open
        let file_result = File::open(apk_file)
            .await;

        let mut apk = match file_result {
            Ok(file) => file,
            Err(err) => {
                error!("[ANDROID_BUILD_VALIDATOR] Error opening/reading file: {}", err);
                result.status = ApkValidationStatus::Unavailable; 
                return result
            }
        };

        // Parse
        let info_result = self.parser.parse_apk_info(&mut apk, APK_V2_BLOCK_ID)
            .await;

        let signature_info = match info_result {
            Ok(info) => {
                result.offsets = Some(info.offsets.clone());
                info
            },
            Err(status) => {
                result.status = status;
                return result
            }
        };

        // Verify
        let verification_result = self.verify_signing(&mut apk, &signature_info, verify_integrity)
            .await;

        match verification_result {
            Ok(info) => {
                result.status = ApkValidationStatus::Success;
                result.data = Some(info)
            },
            Err(err) => {
                result.status = err
            }
        };

        return result;
    }

    async fn verify_signing(
        &self,
        apk: &mut File,
        block_info: &ApkBlockInfo,
        do_verify_integrity: bool,
    ) -> ApkResult<ApkVerificationData> {
        let mut content_digests = HashMap::new();
        let mut signer_certs = Vec::new();

        let mut sig_ref = block_info.signers_block.as_ref();
        let mut signers = get_len_pref_slice(&mut sig_ref)
            .map_err(|_| ApkValidationStatus::InvalidApkFormat)?;

        while !signers.is_empty() {
            if signer_certs.len() > MAX_V2_SIGNERS {
                return Err(ApkValidationStatus::TooManySigners);
            }

            let mut signer = get_len_pref_slice(&mut signers)
                .map_err(|_| ApkValidationStatus::InvalidApkFormat)?;

            let certs = self.verify_signer(&mut signer, &mut content_digests)
                .await?;

            signer_certs.extend(certs);
        }

        if signer_certs.is_empty() {
            return Err(ApkValidationStatus::NoSignersFound);
        }

        if content_digests.is_empty() {
            return Err(ApkValidationStatus::NoDigestFound);
        }

        if do_verify_integrity {
            self.chunker.verify_digest(apk, &content_digests, block_info).await?;
        }

        // TODO LATER
        // let mut verity_root_hash = None;
        // if let Some(verity_digest) = content_digests.get(&ApkChunkDigest::VerifySha256) {
        // let file_len = apk.metadata().await?.len();
        // verity_root_hash = Some(
        //     self.crypt.parse_verity_digest_and_verify_source_length(
        //         verity_digest,
        //         file_len,
        //         signature_info,
        //     )?,
        // );
        // }

        return Ok(
            ApkVerificationData {
                certs: signer_certs,
                digest: content_digests,
            }
        );
    }

    async fn verify_signer(
        &self,
        signer_block: &mut &[u8],
        content_digests: &mut HashMap<ApkChunkDigest, Bytes>,
    ) -> ApkResult<Vec<X509>> {
        let mut signed_data = get_len_pref_slice(signer_block)
            .map_err(|_| ApkValidationStatus::InvalidSignBlockFormat)?;
        let mut signatures = get_len_pref_slice(signer_block)
            .map_err(|_| ApkValidationStatus::InvalidSignBlockFormat)?;
        let public_key_bytes = get_len_pref_slice(signer_block)
            .map_err(|_| ApkValidationStatus::InvalidSignBlockFormat)?;

        let mut best_sig_algo: Option<ApkSignature> = None;
        let mut best_sig = None;
        let mut sig_algorithms = Vec::new();

        while let Ok(signature) = get_len_pref_slice(&mut signatures) {
            if signature.len() < 8 {
                return Err(ApkValidationStatus::InvalidSignBlockFormat);
            }

            let sig_algo = get_u32_le(signature);
            let new_algo = ApkSignature::from(sig_algo)
                .ok_or_else(|| ApkValidationStatus::UnknownSignatureAlgorithm)?;

            sig_algorithms.push(new_algo.clone());

            let is_new_best = best_sig_algo.as_ref()
                .map_or(true, |cur_algo| &new_algo > cur_algo);

            if is_new_best {
                best_sig = Some(
                    get_len_pref_copy(&mut &signature[4..])
                        .map_err(|_| ApkValidationStatus::IncorrectEncryptionData)?
                );
                
                best_sig_algo = Some(new_algo);
            }
        }

        let Some(best_sig_alg) = best_sig_algo else {
            return Err(ApkValidationStatus::SignaturesNotFound)
        };

        let Some(best_sig) = best_sig else {
            return Err(ApkValidationStatus::SignaturesNotFound)
        };

        let digest = best_sig_alg.digest_algo().digest();
        let salt_len = best_sig_alg.salt_len();
        let rsa_padding = best_sig_alg.rsa_padding();

        let pub_key = PKey::public_key_from_der(public_key_bytes)
            .map_err(|_| ApkValidationStatus::IncorrectEncryptionData)?;
        let mut verifier = Verifier::new(digest.clone(), &pub_key)
            .map_err(|_| ApkValidationStatus::IncorrectEncryptionData)?;

        if let Some(len) = salt_len {
            verifier.set_rsa_mgf1_md(digest)
                .map_err(|_| ApkValidationStatus::IncorrectEncryptionData)?;

            verifier.set_rsa_pss_saltlen(RsaPssSaltlen::custom(len))
                .map_err(|_| ApkValidationStatus::IncorrectEncryptionData)?;
        }

        if let Some(padding) = rsa_padding {
            verifier.set_rsa_padding(padding)
                .map_err(|_| ApkValidationStatus::IncorrectEncryptionData)?;
        }

        verifier.update(&signed_data)
            .map_err(|_| ApkValidationStatus::IncorrectEncryptionData)?;

        let is_valid = verifier.verify(best_sig.as_ref())
            .map_err(|_| ApkValidationStatus::IncorrectEncryptionData)?;

        if !is_valid {
            return Err(ApkValidationStatus::InvalidSignature);
        }

        let mut digests = get_len_pref_slice(&mut signed_data)
            .map_err(|_| ApkValidationStatus::UnknownSignatureAlgorithm)?;

        let mut digests_sig_algorithms = Vec::new();
        let mut content_digest = None;

        while let Ok(digest) = get_len_pref_slice(&mut digests) {
            if digest.len() < 8 {
                return Err(ApkValidationStatus::InvalidSignBlockFormat);
            }

            let sig_algorithm = ApkSignature::from(get_u32_le(&digest))
                .ok_or_else(|| ApkValidationStatus::UnknownSignatureAlgorithm)?;

            digests_sig_algorithms.push(sig_algorithm.clone());

            if sig_algorithm == best_sig_alg {
                content_digest = Some(
                    get_len_pref_copy(&mut &digest[4..])
                        .map_err(|_| ApkValidationStatus::InvalidSignBlockFormat)?
                );
            }
        }

        let Some(content_digest) = content_digest else {
            return Err(ApkValidationStatus::NoDigestFound)
        };

        if sig_algorithms != digests_sig_algorithms { // check set?
            return Err(ApkValidationStatus::DigestAndSignatureAlgorithmsMismatch);
        }

        let digest_algo = best_sig_alg.chunk_digest();
        let previous_signer_digest = content_digests.insert(digest_algo.clone(), content_digest.clone());

        if let Some(prev_digest) = previous_signer_digest {
            if prev_digest != content_digest {
                return Err(ApkValidationStatus::PreviousDigestForSameAlgorithmMismatch);
            }
        }

        let mut certificates = get_len_pref_slice(&mut signed_data)
            .map_err(|_| ApkValidationStatus::InvalidSignBlockFormat)?;
        let mut certs = Vec::new();

        while let Ok(encoded_cert) = get_len_pref_copy(&mut certificates) {
            let cert = X509::from_der(&encoded_cert)
                .map_err(|_| ApkValidationStatus::IncorrectEncryptionData)?;

            certs.push(cert);
        }

        if certs.is_empty() {
            return Err(ApkValidationStatus::NoCertificatesFound);
        }

        let cert_pub_key = certs[0].to_der()
            .map_err(|_| ApkValidationStatus::IncorrectEncryptionData)?;

        if public_key_bytes == cert_pub_key.as_slice() {
            return Err(ApkValidationStatus::PubKeyFromCertMismatch);
        }

        let mut additional_attrs = get_len_pref_slice(&mut signed_data)
            .map_err(|_| ApkValidationStatus::InvalidSignBlockFormat)?;

        self.verify_additional_attributes(&mut additional_attrs)?;

        return Ok(certs);
    }

    fn verify_additional_attributes(&self, attrs: &mut &[u8]) -> ApkResult<()> {
        while let Ok(attr) = get_len_pref_slice(attrs) {
            if attr.len() < 4 {
                return Err(ApkValidationStatus::InvalidSignBlockFormat);
            }

            let id = get_u32_le(&&attr);
            match id {
                STRIPPING_PROTECTION_ATTR_ID => {
                    if attr.len() < 8 {
                        return Err(ApkValidationStatus::InvalidSignBlockFormat);
                    }

                    let vers = get_u32_le(&attr[4..]);
                    if vers == SF_ATTRIBUTE_ANDROID_APK_SIGNED_V3_ID {
                        return Err(ApkValidationStatus::InvalidSignBlockFormat);
                    }
                }
                _ => {
                    // not the droid we're looking for, move along, move along.
                }
            }
        }

        return Ok(());
    }
}

use std::collections::HashMap;
use std::path::Path;

use bytes::Bytes;
use openssl::pkey::PKey;
use openssl::sign::{RsaPssSaltlen, Verifier};
use openssl::x509::X509;
use tokio::fs::File;
use tracing::error;

use crate::android::apk::chunker::ApkChunker;
use crate::android::apk::crypto::{ApkChunkDigest, ApkSignature};
use crate::android::apk::parser::{ApkBlockInfo, ApkParser};
use crate::android::apk::reader::{get_len_pref_copy, get_len_pref_slice};
use crate::android::apk::verifier::{ApkVerificationData, ApkVerificationResult};
use codegen_block::status::{ApkResult, ApkValidationStatus};
use core_std::endian::get_u32_le;

const APK_V3_BLOCK_ID: u32 = 0xf05368c0;
const APK_V31_BLOCK_ID: u32 = 0x1b93ad61;

pub struct ApkVerifierV3 {
    parser: ApkParser,
    chunker: ApkChunker,
}

impl Default for ApkVerifierV3 {
    fn default() -> Self {
        Self { parser: ApkParser::default(), chunker: ApkChunker::default() }
    }
}

impl ApkVerifierV3 {

    pub fn new(parser: ApkParser, chunk: ApkChunker) -> Self {
        Self { parser, chunker: chunk }
    }

    pub async fn verify(
        &self,
        apk_file: impl AsRef<Path>,
        verify_integrity: bool,
    ) -> ApkVerificationResult {
        let mut result = ApkVerificationResult::default();

        let file_result = File::open(apk_file).await;
        let mut apk = match file_result {
            Ok(file) => file,
            Err(err) => {
                error!("[ANDROID_BUILD_VALIDATOR] Error opening/reading file: {}", err);
                result.status = ApkValidationStatus::Unavailable;
                return result
            }
        };

        // Try V3.1 first, then fallback to V3
        let info_result_v31 = self.parser.parse_apk_info(&mut apk, APK_V31_BLOCK_ID).await;
        let signature_info = match info_result_v31 {
            Ok(info) => info,
            Err(err) => {
                match err {
                    ApkValidationStatus::SignaturesNotFound => {
                        // fallback to V3
                        match self.parser.parse_apk_info(&mut apk, APK_V3_BLOCK_ID).await {
                            Ok(info) => info,
                            Err(status) => {
                                result.status = status;
                                return result
                            }
                        }
                    }
                    _ => {
                        result.status = err;
                        return result
                    }
                }
            }
        };

        result.offsets = Some(signature_info.offsets.clone());

        let verification_result = self.verify_signing(&mut apk, &signature_info, verify_integrity).await;

        match verification_result {
            Ok(info) => {
                result.status = ApkValidationStatus::Success;
                result.data = Some(info)
            }
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

        let mut signer_count = 0usize;
        while !signers.is_empty() {
            if signer_count > 0 {
                // V3 expects exactly one signer
                return Err(ApkValidationStatus::TooManySigners);
            }

            let mut signer = get_len_pref_slice(&mut signers)
                .map_err(|_| ApkValidationStatus::InvalidSignBlockFormat)?;

            let certs = self.verify_signer(&mut signer, &mut content_digests).await?;
            signer_certs.extend(certs);
            signer_count += 1;
        }

        if signer_count == 0 || signer_certs.is_empty() {
            return Err(ApkValidationStatus::NoSignersFound);
        }

        if content_digests.is_empty() {
            return Err(ApkValidationStatus::NoDigestFound);
        }

        if do_verify_integrity {
            self.chunker.verify_digest(apk, &content_digests, block_info).await?;
        }

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
        let signed_data = get_len_pref_slice(signer_block)
            .map_err(|_| ApkValidationStatus::InvalidSignBlockFormat)?;

        if signer_block.len() < 8 {
            return Err(ApkValidationStatus::InvalidSignBlockFormat);
        }

        let min_sdk_version = get_u32_le(&signer_block[0..4]);
        let max_sdk_version = get_u32_le(&signer_block[4..8]);
        *signer_block = &signer_block[8..];

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

            let is_new_best = best_sig_algo.as_ref().map_or(true, |cur_algo| &new_algo > cur_algo);
            if is_new_best {
                best_sig = Some(
                    get_len_pref_copy(&mut &signature[4..])
                        .map_err(|_| ApkValidationStatus::IncorrectEncryptionData)?
                );
                best_sig_algo = Some(new_algo);
            }
        }

        let Some(best_sig_alg) = best_sig_algo else { return Err(ApkValidationStatus::SignaturesNotFound) };
        let Some(best_sig) = best_sig else { return Err(ApkValidationStatus::SignaturesNotFound) };

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

        verifier.update(signed_data)
            .map_err(|_| ApkValidationStatus::IncorrectEncryptionData)?;
        let is_valid = verifier.verify(best_sig.as_ref())
            .map_err(|_| ApkValidationStatus::IncorrectEncryptionData)?;
        if !is_valid { return Err(ApkValidationStatus::InvalidSignature); }

        let mut signed_data_ref = signed_data;

        let mut digests = get_len_pref_slice(&mut signed_data_ref)
            .map_err(|_| ApkValidationStatus::UnknownSignatureAlgorithm)?;

        let mut digests_sig_algorithms = Vec::new();
        let mut content_digest = None;
        while let Ok(digest_rec) = get_len_pref_slice(&mut digests) {
            if digest_rec.len() < 8 { return Err(ApkValidationStatus::InvalidSignBlockFormat); }
            let sig_algorithm = ApkSignature::from(get_u32_le(&digest_rec))
                .ok_or_else(|| ApkValidationStatus::UnknownSignatureAlgorithm)?;

            digests_sig_algorithms.push(sig_algorithm.clone());
            if sig_algorithm == best_sig_alg {
                content_digest = Some(
                    get_len_pref_copy(&mut &digest_rec[4..])
                        .map_err(|_| ApkValidationStatus::InvalidSignBlockFormat)?
                );
            }
        }

        let Some(content_digest) = content_digest else { return Err(ApkValidationStatus::NoDigestFound) };
        if sig_algorithms != digests_sig_algorithms { return Err(ApkValidationStatus::DigestAndSignatureAlgorithmsMismatch); }

        let digest_algo = best_sig_alg.chunk_digest();
        let previous_signer_digest = content_digests.insert(digest_algo.clone(), content_digest.clone());
        if let Some(prev_digest) = previous_signer_digest {
            if prev_digest != content_digest { return Err(ApkValidationStatus::PreviousDigestForSameAlgorithmMismatch); }
        }

        let mut certificates = get_len_pref_slice(&mut signed_data_ref)
            .map_err(|_| ApkValidationStatus::InvalidSignBlockFormat)?;
        let mut certs = Vec::new();
        while let Ok(encoded_cert) = get_len_pref_copy(&mut certificates) {
            let cert = X509::from_der(&encoded_cert)
                .map_err(|_| ApkValidationStatus::IncorrectEncryptionData)?;
            certs.push(cert);
        }

        if certs.is_empty() { return Err(ApkValidationStatus::NoCertificatesFound); }

        let cert_pub_key_der = certs[0]
            .public_key()
            .and_then(|k| k.public_key_to_der())
            .map_err(|_| ApkValidationStatus::IncorrectEncryptionData)?;
        if public_key_bytes != cert_pub_key_der.as_slice() {
            return Err(ApkValidationStatus::PubKeyFromCertMismatch);
        }

        if signed_data_ref.len() < 8 { return Err(ApkValidationStatus::InvalidSignBlockFormat); }
        let signed_min_sdk = get_u32_le(&signed_data_ref[0..4]);
        let signed_max_sdk = get_u32_le(&signed_data_ref[4..8]);
        if signed_min_sdk != min_sdk_version || signed_max_sdk != max_sdk_version {
            return Err(ApkValidationStatus::InvalidSignBlockFormat);
        }
        let mut rest = &signed_data_ref[8..];
        let _additional_attrs = get_len_pref_slice(&mut rest)
            .map_err(|_| ApkValidationStatus::InvalidSignBlockFormat)?;

        return Ok(certs);
    }
}



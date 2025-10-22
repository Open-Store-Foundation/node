use std::collections::HashMap;
use std::sync::Arc;
use alloy::primitives::{Address, Bytes, B256};
use openssl::hash::MessageDigest;
use openssl::sha::sha256;
use openssl::sign::Verifier;
use openssl::x509::X509;
use codegen_block::status::{ApkResult, ApkValidationStatus};
use codegen_contracts::ext::ToChecksum;
use core_std::finger;
use service_sc::obj::ScObjService;

pub struct ProofVerifier {
    caip2: String,
    protocol_version: u64,
    obj_service: Arc<ScObjService>,
}

impl ProofVerifier {

    pub fn new(caip2: String, protocol_version: u64, obj_service: Arc<ScObjService>) -> Self {
        Self { caip2, protocol_version, obj_service }
    }

    pub fn verify_ownership_proofs_raw(
        &self,
        target: Address,
        fingerprints: &Vec<B256>,
        certs: &Vec<Bytes>,
        proofs: &Vec<Bytes>,
    ) -> ApkValidationStatus {
        let mut ders = Vec::new();
        for der in certs.iter() {
            let result = X509::from_der(der);
            match result {
                Ok(der) => ders.push(der),
                Err(_) => {
                    return ApkValidationStatus::IncorrectCertFormat
                }
            }
        }

        return self.verify_ownership_proofs_cert(target, fingerprints, &ders, proofs)
    }

    pub fn verify_ownership_proofs_cert(
        &self,
        target: Address,
        fingerprints: &Vec<B256>,
        certs: &Vec<X509>,
        proofs: &Vec<Bytes>,
    ) -> ApkValidationStatus {
        let proofs: HashMap<String, &Bytes> = fingerprints.iter().
            zip(proofs)
            .map(|(finger, proof)| {
                (hex::encode_upper(finger.as_slice()), proof)
            })
            .collect();

        let mut cert_to_hash = HashMap::<String, &X509>::new();

        for item in certs {
            let der = item.to_der();

            let Ok(der) = der else {
                return ApkValidationStatus::IncorrectCertFormat;
            };

            let hash = hex::encode_upper(sha256(der.as_slice()));
            cert_to_hash.insert(hash, item);
        }

        let owner_address = target.lower_checksum();
        for (ref pub_key_hash, ref cert) in cert_to_hash {
            let Some(proof) = proofs.get(pub_key_hash) else {
                return ApkValidationStatus::ProofNotFound;
            };

            let Ok(pub_key) = cert.public_key() else {
                return ApkValidationStatus::IncorrectEncryptionData;
            };

            let Ok(mut verifier) = Verifier::new(MessageDigest::sha256(), &pub_key) else {
                return ApkValidationStatus::IncorrectEncryptionData;
            };

            let data = format!("{}::{}::{}", self.caip2, owner_address, finger::sha256(pub_key_hash));
            let Ok(is_valid) = verifier.verify_oneshot(proof, data.as_bytes()) else {
                return ApkValidationStatus::IncorrectEncryptionData;
            };

            if !is_valid {
                return ApkValidationStatus::InvalidProof;
            }
        }

        return ApkValidationStatus::Success;
    }
}
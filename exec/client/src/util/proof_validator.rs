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

pub struct ProofValidator {
    caip2: String,
    protocol_version: u64,
    obj_service: Arc<ScObjService>,
}

impl ProofValidator {

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
            let result = X509::from_der(der.0.as_ref());
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

        let owner_address = target.checksum();
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

#[test]
fn test_verify_ownership_proofs_raw() {
    let data = format!("{}::{}::{}", "eip155:31337", "0x2332a2Be5817F77552969C85d40E19E820334fC3", "21:F9:55:DA:C4:05:CB:A2:1B:BC:26:9A:32:81:64:E0:33:AB:FB:2D:93:EA:53:41:7A:45:81:98:F8:ED:5D:82");

    // 48 130
    let cert_data = hex::decode("3082032830820210020101300d06092a864886f70d01010b0500305a310d300b06035504030c0474657374310d300b060355040b0c0474657374310d300b060355040a0c0474657374310d300b06035504070c0474657374310d300b06035504080c0474657374310d300b0603550406130474657374301e170d3234303131333134313733385a170d3439303130363134313733385a305a310d300b06035504030c0474657374310d300b060355040b0c0474657374310d300b060355040a0c0474657374310d300b06035504070c0474657374310d300b06035504080c0474657374310d300b060355040613047465737430820122300d06092a864886f70d01010105000382010f003082010a0282010100c64d25e48e33fc63024744ddbe50f8cea55f92a0bd4fc81ae9a972a5a1fc9f19314af1054868ea217b472e42193d6c893caa4f88f03d517aec193367cbec7cde24877e76d3a48cb1008a32752879882a17220811a4f264e0dbbf71f1a1134fd8ca42a92c99e3c2b048cc18fd8934e7d601e54bbcdfe335a86432c171b1b210d5d518a6b2df6df5750a5969385e498cf55ad22b842652c5772d67004c8a13282f22398d749423cf1dce4d2ac3b3f7f186103228baba9dc98ae8053d54a8ffe8db48e65b11981142ba211367574a856897efe77093d17e6852fc0b5a5d4cc5835ec635357ae209ff0a91ed8a78e3808998177d441c35774bcaf2737c5285118bab0203010001300d06092a864886f70d01010b05000382010100adc880355126add9e4277d83ded06056b13e34a38136ba1746d70e81504dd2bb5bad7d9f31286d85a1f8c1ae5d2e79939e209081ff8754fe7860fcf0fce05d667120b89784fc25243278bcaee92df71efd07553417275be644180d4b1b272f65cc4a9cc5d28ceb7fa0f76a064b26e599bdafcf676140c60bca0d9ba0185d2031d7533549bf656883f4deca6d1c3edc2ff85c1b76ccc994f1b88a21d58f74118a27fdae26c5a4d74293f16a35cb6b233b8b37f968d1f0e34ff32dbe6e41e4e2eb3829cf18d1da8c9f761871c8556173154f983278d2d8a7869a9f2bbe0b5c6e48fe32507e4fb6ddf73d02e703bc800e3f4466febfec89b50007df4d43c347f529").unwrap();
    let cert = X509::from_der(cert_data.as_slice()).unwrap();

    // 102 211 ... 238
    let proof = hex::decode("66d37bc1082d249680840f7eaaf4f01b34999e452d90cf58b6238269efd6a847a356d488ca6ecdb3ab659446cd0bd9b6e144506c7b7d39c58ef31ec8f1630ab99e2180d18b0d5da18f037cb3c48d659833063a47ed6bade866d9d70f645c4475e634189b8c8dfa03373283d0680c41c9d4150010a16b4055fef6d6cec04e6b188a1e1c36746f89fcb0fdc92ecdced1bf5e261a768ee53694d27f77431572ead2bec5dcc38ba820e64099cf4db33efb669d49013abbab434db374a387757fb583b1a428f34b991fd3cf8ef969eb2be3ededce6974687c04e6a987cb6facf20ad7cdd871c91866a64ba187d0800e1f9065e1e320943f52b88f649e8c6bcceae1ee").unwrap();
    let pub_key = cert.public_key().unwrap();
    let mut verifier = Verifier::new(MessageDigest::sha256(), &pub_key).unwrap();
    let is_valid = verifier.verify_oneshot(proof.as_slice(), data.as_bytes()).unwrap();

    assert!(is_valid);
}

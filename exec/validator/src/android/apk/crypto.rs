#![allow(dead_code)]
#![allow(unused_variables)]

use std::cmp::Ordering;

use openssl::hash::MessageDigest;
use openssl::rsa::Padding;
use ring::digest::{Context, SHA256, SHA512};
use crate::android::apk::crypto::ApkChunkDigest::{Sha256, Sha512, VerifySha256};

pub const SIGNATURE_RSA_PSS_WITH_SHA256: u32 = 0x0101;
pub const SIGNATURE_RSA_PSS_WITH_SHA512: u32 = 0x0102;
pub const SIGNATURE_RSA_PKCS1_V1_5_WITH_SHA256: u32 = 0x0103;
pub const SIGNATURE_RSA_PKCS1_V1_5_WITH_SHA512: u32 = 0x0104;
pub const SIGNATURE_ECDSA_WITH_SHA256: u32 = 0x0201;
pub const SIGNATURE_ECDSA_WITH_SHA512: u32 = 0x0202;
pub const SIGNATURE_DSA_WITH_SHA256: u32 = 0x0301;
pub const SIGNATURE_VERITY_RSA_PKCS1_V1_5_WITH_SHA256: u32 = 0x0421;
pub const SIGNATURE_VERITY_ECDSA_WITH_SHA256: u32 = 0x0423;
pub const SIGNATURE_VERITY_DSA_WITH_SHA256: u32 = 0x0425;

pub const CONTENT_DIGEST_CHUNKED_SHA256: u32 = 1;
pub const CONTENT_DIGEST_CHUNKED_SHA512: u32 = 2;
pub const CONTENT_DIGEST_VERITY_CHUNKED_SHA256: u32 = 3;

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum ApkSignature {
    RsaPssWithSha256,
    RsaPssWithSha512,
    RsaPkcs1V15WithSha256,
    RsaPkcs1V15WithSha512,
    EcdsaWithSha256,
    EcdsaWithSha512,
    DsaWithSha256,
    VerityRsaPkcs1V15WithSha256,
    VerityEcdsaWithSha256,
    VerityDsaWithSha256,
}

pub enum ApkSigDigest {
    Sha256,
    Sha512
}

#[derive(PartialEq, Hash, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub enum ApkChunkDigest {
    Sha256,
    VerifySha256,
    Sha512,
}

impl ApkSigDigest {
    pub fn digest(&self) -> MessageDigest {
        return match self {
            ApkSigDigest::Sha256 => MessageDigest::sha256(),
            ApkSigDigest::Sha512 => MessageDigest::sha512(),
        }
    }
}

impl ApkSignature {
    pub fn from(
        sig_algorithm: u32
    ) -> Option<ApkSignature> {
        return match sig_algorithm {
            SIGNATURE_RSA_PSS_WITH_SHA256 => Some(Self::RsaPssWithSha256),
            SIGNATURE_RSA_PSS_WITH_SHA512 => Some(Self::RsaPssWithSha512),

            SIGNATURE_RSA_PKCS1_V1_5_WITH_SHA256 => Some(Self::RsaPkcs1V15WithSha256),
            SIGNATURE_RSA_PKCS1_V1_5_WITH_SHA512 => Some(Self::RsaPkcs1V15WithSha512),

            SIGNATURE_ECDSA_WITH_SHA256 => Some(Self::EcdsaWithSha256),
            SIGNATURE_ECDSA_WITH_SHA512 => Some(Self::EcdsaWithSha512),

            SIGNATURE_DSA_WITH_SHA256 => Some(Self::DsaWithSha256),

            SIGNATURE_VERITY_RSA_PKCS1_V1_5_WITH_SHA256 => Some(Self::VerityRsaPkcs1V15WithSha256),
            SIGNATURE_VERITY_ECDSA_WITH_SHA256 => Some(Self::EcdsaWithSha256),
            SIGNATURE_VERITY_DSA_WITH_SHA256 => Some(Self::DsaWithSha256),
            _ => None,
        };
    }

    pub fn is_supported(
        sig_algorithm: u32
    ) -> bool {
        return matches!(
            sig_algorithm,
            SIGNATURE_RSA_PSS_WITH_SHA256
                | SIGNATURE_RSA_PSS_WITH_SHA512
                | SIGNATURE_RSA_PKCS1_V1_5_WITH_SHA256
                | SIGNATURE_RSA_PKCS1_V1_5_WITH_SHA512

                | SIGNATURE_ECDSA_WITH_SHA256
                | SIGNATURE_ECDSA_WITH_SHA512

                | SIGNATURE_DSA_WITH_SHA256

                | SIGNATURE_VERITY_RSA_PKCS1_V1_5_WITH_SHA256
                | SIGNATURE_VERITY_ECDSA_WITH_SHA256
                | SIGNATURE_VERITY_DSA_WITH_SHA256
        );
    }

    pub fn salt_len(&self) -> Option<i32> {
        return match self {
            Self::RsaPssWithSha256 => Some(32),
            Self::RsaPssWithSha512 => Some(64),
            _ => None
        }
    }
    
    pub fn rsa_padding(&self) -> Option<Padding> {
        return match self {
            // RSA-PSS variants
            Self::RsaPssWithSha256
            | Self::RsaPssWithSha512 => Some(Padding::PKCS1_PSS),

            // RSA PKCS#1 v1.5 variants
            Self::RsaPkcs1V15WithSha256
            | Self::RsaPkcs1V15WithSha512
            | Self::VerityRsaPkcs1V15WithSha256 => Some(Padding::PKCS1),

            // ECDSA and DSA do not use RSA padding at all
            Self::EcdsaWithSha256
            | Self::EcdsaWithSha512
            | Self::DsaWithSha256
            | Self::VerityEcdsaWithSha256
            | Self::VerityDsaWithSha256 => None,
        }
    }

    pub fn name(&self) -> &'static str {
        return match self {
            Self::RsaPssWithSha256 => "SHA256withRSA/PSS",
            Self::RsaPssWithSha512 => "SHA512withRSA/PSS",
            Self::RsaPkcs1V15WithSha256 | Self::VerityRsaPkcs1V15WithSha256 => "SHA256withRSA",
            Self::RsaPkcs1V15WithSha512 => "SHA512withRSA",
            Self::EcdsaWithSha256 | Self::VerityEcdsaWithSha256 => "SHA256withECDSA",
            Self::DsaWithSha256 | Self::VerityDsaWithSha256 => "SHA256withDSA",
            Self::EcdsaWithSha512 => "SHA512withECDSA",
        }
    }

    pub fn digest_algo(&self) -> ApkSigDigest {
        return match self {
            Self::RsaPssWithSha256
            | Self::RsaPkcs1V15WithSha256
            | Self::EcdsaWithSha256
            | Self::DsaWithSha256
            | Self::VerityRsaPkcs1V15WithSha256
            | Self::VerityEcdsaWithSha256
            | Self::VerityDsaWithSha256 => ApkSigDigest::Sha256,

            Self::RsaPssWithSha512
            | Self::RsaPkcs1V15WithSha512
            | Self::EcdsaWithSha512  => ApkSigDigest::Sha512,
        }
    }

    pub fn chunk_digest(&self) -> ApkChunkDigest {
        return match self {
            Self::RsaPssWithSha256
            | Self::RsaPkcs1V15WithSha256
            | Self::EcdsaWithSha256
            | Self::DsaWithSha256  => Sha256,

            Self::VerityRsaPkcs1V15WithSha256
            | Self::VerityEcdsaWithSha256
            | Self::VerityDsaWithSha256 => VerifySha256,

            Self::RsaPssWithSha512
            | Self::RsaPkcs1V15WithSha512
            | Self::EcdsaWithSha512  => Sha512,
        }
    }
}

impl PartialOrd for ApkSignature {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ApkSignature {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_digest = self.chunk_digest();
        let other_digest = other.chunk_digest();
        self_digest.cmp(&other_digest)
    }
}

impl ApkChunkDigest {
    pub fn chunk_digest(&self) -> ApkSigDigest {
        return match self {
            Sha256 | VerifySha256 => ApkSigDigest::Sha256,
            Sha512 => ApkSigDigest::Sha512,
        }
    }

    pub fn context(&self) -> Context {
        return match self.chunk_digest() {
            ApkSigDigest::Sha256 => Context::new(&SHA256),
            ApkSigDigest::Sha512 => Context::new(&SHA512),
        }
    }

    pub fn digest_size(
        &self
    ) -> usize {
        return match self {
            Sha256 | VerifySha256 => 32,
            Sha512 => 64,
        }
    }
}


// #[derive(Debug)]
// struct VerityResult {
//     root_hash: Vec<u8>,
//     // Add other fields from VerityResult if needed
// }
//
// #[derive(Debug)]
// struct VerbatimX509Certificate {
//     // Replace with appropriate fields for Rust
// }
//
// #[derive(Debug)]
// struct VerifiedProofOfRotation {
//     // Replace with appropriate fields for Rust
// }
//
// pub fn set_unsigned_int32_little_endian(value: u32, result: &mut [u8], offset: usize) {
//     LittleEndian::write_u32(&mut result[offset..offset + 4], value);
// }


// struct MultipleDigestDataDigester<'a> {
//     mds: &'a mut [Context],
// }
//
// impl<'a> MultipleDigestDataDigester<'a> {
//     pub fn new(mds: &'a mut [Context]) -> Self {
//         MultipleDigestDataDigester { mds }
//     }
// }
//
// impl<'a> DataDigester for MultipleDigestDataDigester<'a> {
//     pub fn consume(&mut self, buffer: &[u8]) -> Result<(), ApkSigningError> {
//         for md in self.mds.iter_mut() {
//             md.update(buffer);
//         }
//
//         Ok(())
//     }
// }

// pub fn verify_proof_of_rotation_struct(
//     por_buf: &[u8],
// ) -> Result<VerifiedProofOfRotation, ApkSigningError> {
//     let mut level_count = 0;
//     let mut last_sig_algorithm = -1;
//     let mut last_cert: Option<VerbatimX509Certificate> = None;
// }
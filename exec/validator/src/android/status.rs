use thiserror::Error;

#[derive(Error, Debug, Default, PartialEq, Eq, Clone)]
pub enum ApkValidationStatus {
    #[error("Unavailable")]
    #[default]
    Unavailable,

    #[error("Success")]
    Success,

    // #[error("InvalidRequestData")]
    // InvalidRequestData,
    // #[error("RetrieveObjInfoFailed")]
    // RetrieveObjInfo,
    // #[error("DownloadObj")]
    // DownloadObj,

    #[error("HashMismatch")]
    HashMismatch,
    #[error("InvalidApkFormat")]
    InvalidApkFormat,
    #[error("InvalidSignBlockFormat")]
    InvalidSignBlockFormat,
    #[error("Zip64NotSupported")]
    Zip64NotSupported,

    #[error("TooManySigners")]
    TooManySigners,
    #[error("NoSignersFound")]
    NoSignersFound,
    #[error("NoDigestFound")]
    NoDigestFound,

    #[error("UnknownSignatureAlgorithm")]
    UnknownSignatureAlgorithm,
    #[error("SignaturesNotFound")]
    SignaturesNotFound,
    #[error("IncorrectEncryptionData")]
    IncorrectEncryptionData,
    #[error("InvalidSignature")]
    InvalidSignature,

    #[error("DigestAndSignatureAlgorithmsMismatch")]
    DigestAndSignatureAlgorithmsMismatch,
    #[error("PreviousDigestForSameAlgorithmMismatch")]
    PreviousDigestForSameAlgorithmMismatch,

    #[error("NoCertificatesFound")]
    NoCertificatesFound,
    #[error("PubKeyFromCertMismatch")]
    PubKeyFromCertMismatch,

    #[error("NoKnownDigestToCheck")]
    NoKnownDigestToCheck,
    #[error("DigestMismatch")]
    DigestMismatch,
    #[error("TooManyChunks")]
    TooManyChunks,
    #[error("DigestAlgorithmNotFound")]
    DigestAlgorithmNotFound,

    #[error("ProofNotFound")]
    ProofNotFound,
    #[error("IncorrectEncryptionData")]
    IncorrectCertFormat,
    #[error("InvalidProof")]
    InvalidProof,
}

impl ApkValidationStatus {
    pub fn code(&self) -> u32 {
        match self {
            ApkValidationStatus::Unavailable => 0,
            ApkValidationStatus::Success => 1,

            // ApkValidationStatus::InvalidRequestData => 2,
            // ApkValidationStatus::RetrieveObjInfo => 3,
            // ApkValidationStatus::DownloadObj => 3,

            ApkValidationStatus::InvalidApkFormat => 10,
            ApkValidationStatus::InvalidSignBlockFormat => 11,
            ApkValidationStatus::Zip64NotSupported => 12,
            ApkValidationStatus::HashMismatch => 13,

            ApkValidationStatus::TooManySigners => 20,
            ApkValidationStatus::NoSignersFound => 21,
            ApkValidationStatus::NoDigestFound => 22,

            ApkValidationStatus::UnknownSignatureAlgorithm => 30,
            ApkValidationStatus::IncorrectEncryptionData => 31,
            ApkValidationStatus::SignaturesNotFound => 32,
            ApkValidationStatus::InvalidSignature => 33,

            ApkValidationStatus::DigestAndSignatureAlgorithmsMismatch => 40,
            ApkValidationStatus::PreviousDigestForSameAlgorithmMismatch => 41,

            ApkValidationStatus::NoCertificatesFound => 50,
            ApkValidationStatus::PubKeyFromCertMismatch => 51,

            ApkValidationStatus::NoKnownDigestToCheck => 60,
            ApkValidationStatus::DigestMismatch => 61,
            ApkValidationStatus::TooManyChunks => 62,
            ApkValidationStatus::DigestAlgorithmNotFound => 63,

            ApkValidationStatus::ProofNotFound => 70,
            ApkValidationStatus::IncorrectCertFormat => 71,
            ApkValidationStatus::InvalidProof => 75,
        }
    }
}

pub type ApkResult<T> = Result<T, ApkValidationStatus>;

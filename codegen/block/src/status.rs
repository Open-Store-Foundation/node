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
    IncorrectEncryptionData1,
    #[error("VersionIsOutdated")]
    VersionIsOutdated,
    #[error("AssetlinkIsNotVerified")]
    AssetlinkIsNotVerified,
    #[error("PublicKeyFormat")]
    PublicKeyFormat,
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
            ApkValidationStatus::IncorrectEncryptionData1 => 71,
            ApkValidationStatus::VersionIsOutdated => 72,
            ApkValidationStatus::AssetlinkIsNotVerified => 73,
            ApkValidationStatus::PublicKeyFormat => 74,
            ApkValidationStatus::InvalidProof => 75,
        }
    }
}

impl From<i32> for ApkValidationStatus {
    fn from(value: i32) -> Self {
        match value {
            1 => Self::Success,

            10 => Self::InvalidApkFormat,
            11 => Self::InvalidSignBlockFormat,
            12 => Self::Zip64NotSupported,

            20 => Self::TooManySigners,
            21 => Self::NoSignersFound,
            22 => Self::NoDigestFound,

            30 => Self::UnknownSignatureAlgorithm,
            31 => Self::IncorrectEncryptionData,
            32 => Self::SignaturesNotFound,
            33 => Self::InvalidSignature,

            40 => Self::DigestAndSignatureAlgorithmsMismatch,
            41 => Self::PreviousDigestForSameAlgorithmMismatch,

            50 => Self::NoCertificatesFound,
            51 => Self::PubKeyFromCertMismatch,

            60 => Self::NoKnownDigestToCheck,
            61 => Self::DigestMismatch,
            62 => Self::TooManyChunks,
            63 => Self::DigestAlgorithmNotFound,

            70 => Self::ProofNotFound,
            71 => Self::IncorrectEncryptionData1,
            72 => Self::VersionIsOutdated,
            73 => Self::AssetlinkIsNotVerified,
            74 => Self::PublicKeyFormat,
            75 => Self::InvalidProof,

            // Default case, including 0
            _ => Self::Unavailable,
        }
    }
}

pub type ApkResult<T> = Result<T, ApkValidationStatus>;
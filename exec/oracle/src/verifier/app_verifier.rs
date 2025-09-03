use async_trait::async_trait;
use crate::verifier::android::app_verifier::AppVerificationResult;

#[async_trait]
pub trait AppVerifier {
    async fn verify(
        &self,
        app_package: String,
        owner_version: u64,
        website: String,
        fingerprints: Vec<String>,
    ) -> AppVerificationResult;
}

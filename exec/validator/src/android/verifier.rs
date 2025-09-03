use std::path::Path;
use crate::android::apk::verifier::ApkVerificationResult;

pub trait ApkBuildVerifier {
    async fn verify(
        &self,
        build_file: impl AsRef<Path>,
    ) -> ApkVerificationResult;
}

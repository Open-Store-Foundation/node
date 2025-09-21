use crate::android::apk::verifier::{ApkVerificationResult, ApkVerifierV2};
use std::path::Path;
use std::sync::Arc;
use core_log::init_tracer;
use core_std::arc;
use crate::android::apk::chunker::ApkChunker;
use crate::android::apk::parser::ApkParser;
use crate::android::apk::verifier_v3::ApkVerifierV3;
use codegen_block::status::ApkValidationStatus;
use crate::android::verifier::ApkBuildVerifier;

pub struct AndroidBuildVerifier {
    verifier: Arc<ApkVerifierV2>,
    verifier_v3: Arc<ApkVerifierV3>,
}

impl AndroidBuildVerifier {
    pub fn new(verifier: Arc<ApkVerifierV2>, verifier_v3: Arc<ApkVerifierV3>) -> AndroidBuildVerifier {
        return AndroidBuildVerifier {
            verifier,
            verifier_v3,
        };
    }
}

impl ApkBuildVerifier for AndroidBuildVerifier {

    async fn verify(
        &self,
        build_file: impl AsRef<Path>,
    ) -> ApkVerificationResult {
        // Parse cert
        let result = self.verifier_v3.verify(build_file.as_ref(), true).await;
        if result.status != ApkValidationStatus::SignaturesNotFound {
            return result;
        }

        let data = self.verifier.verify(build_file, true)
            .await;

        return data
    }
}

#[tokio::test]
async fn test() {
    let _guard = init_tracer();

    let verifier = arc!(ApkVerifierV2::new(ApkParser::default(), ApkChunker::default()));
    let verifier_v3 = arc!(ApkVerifierV3::new(ApkParser::default(), ApkChunker::default()));
    let build_verifier = arc!(AndroidBuildVerifier::new(verifier.clone(), verifier_v3.clone()));
    let result = build_verifier.verify(
        "/Users/andrew/Downloads/app-release-unsigned-signed.apk",
    ).await;

    println!("Result: {:?}", result)
}

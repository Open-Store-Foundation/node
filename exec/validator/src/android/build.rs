use crate::android::apk::verifier::{ApkVerificationResult, ApkVerifierV2};
use std::path::Path;
use std::sync::Arc;
use core_log::init_tracer;
use core_std::arc;
use crate::android::apk::chunker::ApkChunker;
use crate::android::apk::parser::ApkParser;
use crate::android::verifier::ApkBuildVerifier;

pub struct AndroidBuildVerifier {
    verifier: Arc<ApkVerifierV2>,
}

impl AndroidBuildVerifier {
    pub fn new(apk_parser: &Arc<ApkVerifierV2>) -> AndroidBuildVerifier {
        return AndroidBuildVerifier {
            verifier: apk_parser.clone(),
        };
    }
}

impl ApkBuildVerifier for AndroidBuildVerifier {

    async fn verify(
        &self,
        build_file: impl AsRef<Path>,
    ) -> ApkVerificationResult {
        // Parse cert
        let data = self.verifier.verify(build_file, false)
            .await;

        return data
    }
}

#[tokio::test]
async fn test() {
    init_tracer();

    let verifier = arc!(ApkVerifierV2::new(ApkParser::default(), ApkChunker::default()));
    let build_verifier = arc!(AndroidBuildVerifier::new(&verifier));
    let result = build_verifier.verifier.verify(
        "../examples/android/app/build/outputs/apk/release/app-release.apk",
        true,
    ).await;

    println!("Result: {:?}", result)
}

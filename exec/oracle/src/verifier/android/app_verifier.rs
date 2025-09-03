use crate::verifier::app_verifier::AppVerifier;
use crate::verifier::asset_provider::{AssetData, AssetLink, AssetProvider};
use alloy::primitives::Address;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use core_std::empty::Empty;
use net_client::node::result::EthResult;
use openssl::hash::MessageDigest;
use openssl::pkey::PKey;
use openssl::rsa::Padding;
use openssl::sign::Verifier;
use openssl::x509::X509;
use service_sc::assetlinks::{ScAssetLinkService, AssetlinkStatusCode};
use service_sc::obj::ScObjService;
use std::convert::Into;
use std::fs::File;
use std::panic::catch_unwind;
use std::path::Path;
use std::str;
use std::str::FromStr;
use std::sync::Arc;
use async_trait::async_trait;
use tokio::fs;
use tokio::process::Command;
use tracing::{error, info, warn};
use client_tg::tg_alert;
use core_std::finger;
use core_std::trier::SyncTrier;

pub struct AndroidAppVerifier {
    pub assets: Arc<AssetProvider>,
    pub assetlinks: Arc<ScAssetLinkService>,
    pub app: Arc<ScObjService>,
}

impl AndroidAppVerifier {
    pub fn new(assets: &Arc<AssetProvider>, assetlinks: &Arc<ScAssetLinkService>, app: &Arc<ScObjService>) -> Self {
        Self { assets: assets.clone(), assetlinks: assetlinks.clone(), app: app.clone() }
    }
}

pub struct AppVerificationResult {
    pub version: u64,
    pub new_status: AssetlinkStatusCode
}

impl AppVerificationResult {
    pub fn current( version: u64) -> Self {
        Self { version, new_status: AssetlinkStatusCode::Undefined }
    }
}

#[async_trait]
impl AppVerifier for AndroidAppVerifier {

    async fn verify(
        &self,
        app_package: String,
        owner_version: u64,
        website: String,
        fingerprints: Vec<String>,
    ) -> AppVerificationResult {
        let mut result = AppVerificationResult::current(owner_version);

        let fingerprints: Vec<String> = fingerprints.iter()
            .map(|finger| finger::sha256(finger))
            .collect();

        info!("[ORACLE_VERIFIER] Cert fingerprint: {:?}", &fingerprints);

        let mut assets: Option<AssetData> = None;
        let mut trier = SyncTrier::new(5, 1.0, 4);

        // Get assets
        while trier.iterate().await {
            let assets_result = self.assets.get_assets(&website)
                .await;

            match assets_result {
                Ok(data) => {
                    assets = Some(data);
                    break;
                },
                Err(error) => {
                    result.new_status = error;

                    if error == AssetlinkStatusCode::UnreachableLinkError && !trier.is_last() {
                        warn!("[ORACLE_VERIFIER] Can't reach out to website {}, with status {}, schedule retry!", &website, error);
                        continue;
                    } else {
                        warn!("[ORACLE_VERIFIER] Can't reach out to website {}, with status {}!", &website, error);
                        return result
                    }
                },
            };
        }

        let Some(assets) = assets else {
            tg_alert!(format!("Assets is EMPTY for {}", website));
            error!("[ORACLE_VERIFIER] Asset is EMPTY for {}!", &website);
            return result;
        };

        let links: Vec<&AssetLink> = assets.links.iter()
            .filter(|item| {
                let package = item.target.package_name.clone()
                    .or_empty();

                package == app_package && item.target.is_android()
            })
            .collect();

        if links.is_empty() {
            warn!("[ORACLE_VERIFIER] Website {} doesn't contains package {}!", &website, app_package);
            result.new_status = AssetlinkStatusCode::NoPackageError;
            return result;
        }

        info!("[ORACLE_VERIFIER] Assets: {:?}", &links);

        // Verify fingerprint
        let has_fingerprint = links.iter()
            .any(|item| {
                item.target.fingerprints
                    .as_ref()
                    .map_or(false, |val| {
                        fingerprints.iter().all(|fingerprint| {
                            val.iter().any(|print| {
                                print.eq_ignore_ascii_case(fingerprint)
                            })
                        })
                    })
            });

        if !has_fingerprint {
            warn!("[ORACLE_VERIFIER] Website {} doesn't contains fingerprints!", &website);
            result.new_status = AssetlinkStatusCode::NoFingerprintError;
            return result;
        }

        info!("[ORACLE_VERIFIER] Website {} contain fingerprint!", &website);

        result.new_status = AssetlinkStatusCode::Success;
        return result;
    }
}

#[cfg(test)]
mod tests {
    use core_log::init_tracer;

    #[tokio::test]
    async fn test_app_verifier() {
        init_tracer();

        // let url =  EnvApp::node_url();
        // let confirm =  EnvApp::confirmations();
        //
        // let http = HttpProvider::http_client();
        // let web3 = NodeProvider::web3(url, &http).expect("");
        // let pk = PkProvider::default();
        // let eth = arc!(EthClient::new(web3, &pk, confirm));
        //
        // let asset_provider: Arc<ScAssetLinkProvider> = arc!(ScAssetLinkProvider::new(config::assetlink_address(), &eth));
        // let app_provider = arc!(ScAppProvider::new(&eth));
        // let assets = arc!(AssetProvider::new(&http));
        //
        // let app_verifier = arc!(AndroidAppVerifier::new(&assets, &asset_provider, &app_provider));
    }
}

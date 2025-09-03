use std::ops::Add;

use net_client::http::HttpClient;
use serde::{Deserialize, Serialize};
use tracing::error;
use url::Url;
use service_sc::assetlinks::AssetlinkStatusCode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Target {
    pub namespace: String,
    pub package_name: Option<String>,
    #[serde(rename = "sha256_cert_fingerprints")]
    pub fingerprints: Option<Vec<String>>,
}

impl Target {
    const HTTPS_SCHEME: &'static str = "https";
    const ANDROID_APP: &'static str = "android_app";

    pub fn is_android(&self) -> bool {
        self.namespace.eq(Target::ANDROID_APP)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetLink {
    pub target: Target,
}

#[derive(Debug)]
pub struct AssetData {
    pub links: Vec<AssetLink>,
}

pub struct AssetProvider {
    pub client: HttpClient,
}

impl AssetProvider {

    const WELL_KNOW_PATH: &'static str = ".well-known/assetlinks.json";

    pub fn new(client: &HttpClient) -> Self {
        Self { client: client.clone() }
    }
    
    pub async fn get_assets(&self, website: &String) -> Result<AssetData, AssetlinkStatusCode> {
        let assets_url = self.build_assets_link(website)?;
        
        let response = self.client.get(assets_url)
            .send()
            .await
            .map_err(|e| {
                error!("[ASSET_PROVIDER] Can't reach out the website: {}, {} - {}", website, AssetProvider::WELL_KNOW_PATH, e);
                AssetlinkStatusCode::UnreachableLinkError
            })?;

        let data = response.bytes()
            .await
            .map_err(|e| {
                error!("[ASSET_PROVIDER] Can't read the website: {}, {}", website, AssetProvider::WELL_KNOW_PATH);
                AssetlinkStatusCode::ContentReadingError
            })?;

        let result = serde_json::from_slice(data.as_ref())
            .map_err(|e| {
                error!("[ASSET_PROVIDER] Can't parse the website: {}, {}", website, AssetProvider::WELL_KNOW_PATH);
                AssetlinkStatusCode::AssetlinkFormatError
            })?;

        return Ok(
            AssetData { links: result }
        );
    }
    
    pub fn build_assets_link(&self, website: &String) -> Result<Url, AssetlinkStatusCode> {
        let mut url = website.clone();

        if !url.ends_with("/") {
            url = url.add("/"); // TODO https://github.com/servo/rust-url/pull/934
        }

        let origin_url = Url::parse(&url)
            .map_err(|e| {
                error!("[ASSET_PROVIDER] Can't parse url: {}, {}", url, e);
                AssetlinkStatusCode::UrlFormatError
            })?;

        if !origin_url.scheme().eq_ignore_ascii_case(Target::HTTPS_SCHEME) {
            error!("[ASSET_PROVIDER] Scheme should be https: {} - {}", origin_url.scheme(), origin_url);
            return Err(AssetlinkStatusCode::WebsiteFormatError);
        }

        if origin_url.host().is_none() {
            error!("[ASSET_PROVIDER] Host shouldn't be empty: {}", origin_url);
            return Err(AssetlinkStatusCode::WebsiteFormatError);
        }

        let origin_path = origin_url.path();
        if origin_path.len() == 1 && !origin_path.eq("/") {
            error!("[ASSET_PROVIDER] Path should be empty: {}", origin_url);
            return Err(AssetlinkStatusCode::WebsiteFormatError);
        }

        if origin_path.len() > 1 {
            error!("[ASSET_PROVIDER] Path should be empty: {}", origin_url);
            return Err(AssetlinkStatusCode::WebsiteFormatError);
        }

        if origin_url.query().is_some() {
            error!("[ASSET_PROVIDER] Query must be empty: {}", origin_url);
            return Err(AssetlinkStatusCode::WebsiteFormatError);
        }

        let assets_url = origin_url.join(AssetProvider::WELL_KNOW_PATH)
            .map_err(|_| {
                error!("[ASSET_PROVIDER] Can't join url: {}, {}", url, AssetProvider::WELL_KNOW_PATH);
                AssetlinkStatusCode::WebsiteFormatError
            })?;


        return Ok(assets_url);
    }
}

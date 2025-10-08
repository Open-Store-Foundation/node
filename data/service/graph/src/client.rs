use net_client::http::HttpClient;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub enum GraphError {
    #[error("URL parsing error: {0}")]
    UrlParse(#[from] url::ParseError),

    #[error("HTTP client error: {0}")]
    HttpClient(String),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Unexpected response structure")]
    Unexpected,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppAsset {
    pub id: String,
    pub appId: String,
    pub name: String,
    pub description: String,
    pub protocolId: i32,
    pub categoryId: i32,
    pub platformId: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct GraphData {
    apps: Vec<AppAsset>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct GraphResponse {
    data: Option<GraphData>,
    errors: Option<serde_json::Value>,
}

pub struct GraphClient {
    client: HttpClient,
    base_url: Url,
}

const GET_UPDATE_QUERY: &str = "query GetAppAssets($updatedAtBlock: BigInt, $first: Int!, $afterId: ID) { apps: appAssets(where: { updatedAtBlock_gte: $updatedAtBlock, id_gt: $afterId }, orderBy: id, orderDirection: asc, first: $first) { id appId name description protocolId categoryId platformId } }";

impl GraphClient {
    pub fn new(client: HttpClient, base_url: Url) -> Self {
        Self { client, base_url }
    }

    pub async fn fetch_app_assets_since(&self, updated_at_block_gte: u64) -> Result<Vec<AppAsset>, GraphError> {
        let mut all = Vec::new();
        let mut last_id: Option<String> = None;

        loop {
            let variables = serde_json::json!({
                "updatedAtBlock": updated_at_block_gte.to_string(),
                "first": 1000,
                "afterId": last_id,
            });

            let body = serde_json::json!({ "query": GET_UPDATE_QUERY, "variables": variables });

            let res = self.client
                .post(self.base_url.as_str())
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await?;

            if !res.status().is_success() {
                return Err(GraphError::HttpClient(res.status().to_string()));
            }

            let resp: GraphResponse = res.json().await?;
            let mut items = resp.data.ok_or(GraphError::Unexpected)?.apps;
            if items.is_empty() {
                break;
            }

            last_id = items.last().map(|i| i.id.clone());
            all.append(&mut items);

            if last_id.is_none() {
                break;
            }
        }

        Ok(all)
    }
}



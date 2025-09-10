use tracing::info;
use crate::error::EthScanError;
use crate::models::{GetLogsParams, LogsResponse};
use net_client::http::HttpClient;
use url::Url;

pub struct EthScanClient {
    client: HttpClient,
    chain_id: String,
    api_key: String,
    base_url: String,
}

impl EthScanClient {
    
    pub fn new(client: HttpClient, chain_id: u64, api_key: String) -> Self {
        Self {
            client,
            chain_id: chain_id.to_string(),
            api_key,
            base_url: "https://api.etherscan.io/v2/api".to_string(),
        }
    }

    pub async fn get_logs(&self, params: &GetLogsParams) -> Result<LogsResponse, EthScanError> {
        let mut url = Url::parse(&self.base_url)?;

        url.query_pairs_mut()
            .append_pair("chainid", &self.chain_id)
            .append_pair("module", "logs")
            .append_pair("action", "getLogs")
            .append_pair("fromBlock", &params.from_block.to_string())
            .append_pair("apikey", &self.api_key);

        if let Some(ref topic0) = params.topic0 {
            url.query_pairs_mut()
                .append_pair("topic0", topic0);
        }

        if let Some(ref address) = params.address {
            url.query_pairs_mut()
                .append_pair("address", address);
        }

        if let Some(ref page) = params.page {
            url.query_pairs_mut()
                .append_pair("page", &page.to_string());
        }

        if let Some(ref offset) = params.offset {
            url.query_pairs_mut()
                .append_pair("offset", &offset.to_string());
        }
        
        info!("Requesting logs from EthScan: {:?}", url.to_string());
        let logs_response = self.client
            .get(&url.to_string())
            .send()
            .await?
            .json::<LogsResponse>()
            .await?;

        if logs_response.status != "0" && logs_response.status != "1" {
            return Err(EthScanError::ApiError {
                status: logs_response.status,
                message: logs_response.message,
            });
        }
        
        Ok(logs_response)
    }
}

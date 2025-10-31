use reqwest::StatusCode;
use tracing::{error, info};
use crate::error::EthScanError;
use crate::models::{GetLogsParams, LogsResponse};
use net_client::http::HttpClient;
use url::Url;
use core_std::trier::SyncTrier;

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
        let mut sync = SyncTrier::new(3, 1.0, 2); // TODO use tower
        let mut url = Url::parse(&self.base_url)?;

        url.query_pairs_mut()
            .append_pair("chainid", &self.chain_id)
            .append_pair("module", "logs")
            .append_pair("action", "getLogs")
            .append_pair("fromBlock", &params.from_block.to_string())
            .append_pair("apikey", &self.api_key);

        if let Some(ref to_block) = params.to_block {
            url.query_pairs_mut()
                .append_pair("toBlock", &to_block.to_string());
        }
        
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

        loop {
            sync.iterate().await;

            info!("Requesting logs from EthScan: {:?}", url.to_string());

            let result = self.client
                .get(&url.to_string())
                .send()
                .await?;

            if result.status() != StatusCode::OK {
                if sync.is_last() {
                    return Err(EthScanError::ApiError {
                        status: result.status().to_string(),
                        message: result.text().await?,
                    });
                }

                error!(
                    "Error while requesting logs from EthScan: {}. Response: {}",
                    result.status(),
                    result.text().await?
                );

                continue;
            }

            let log_response = result.json::<LogsResponse>()
                .await?;

            if log_response.status != "0" && log_response.status != "1" {
                return Err(EthScanError::ApiError {
                    status: log_response.status,
                    message: log_response.message,
                });
            }

            return Ok(log_response)
        }
    }
}

use crate::error::EventError;
use alloy::primitives::{Address, B256};
use alloy::providers::Provider;
use alloy::rpc::types::Filter;
use net_client::node::provider::Web3Provider;
use client_ethscan::client::EthScanClient;
use client_ethscan::models::{GetLogsParams, LogsResponse};
use std::sync::Arc;

pub enum EventLogClient {
    Eth(Arc<Web3Provider>),
    EthScan(Arc<EthScanClient>),
}

pub struct EventLogService {
    client: EventLogClient,
}

impl EventLogService {

    pub fn new(client: EventLogClient) -> Self {
        Self { client }
    }

    pub async fn get_logs(&self, params: &GetLogsParams) -> Result<LogsResponse, EventError> {
        match self.client {
            EventLogClient::Eth(ref client) => {
                let mut filter = Filter::new()
                    .from_block(params.from_block);

                if let Some(ref addr) = params.address {
                    if let Ok(address) = Address::parse_checksummed(addr.as_str(), None) {
                        filter = filter.address(address);
                    }
                }

                if let Some(ref topic0) = params.topic0 {
                    if let Ok(sig) = topic0.parse::<B256>() {
                        filter = filter.event_signature(sig);
                    }
                }

                if let Some(to) = params.to_block {
                    filter = filter.to_block(to);
                }

                let logs = client.get_logs(&filter)
                    .await
                    .map_err(|e| EventError::Rpc(e.to_string()))?;

                let resp = LogsResponse {
                    status: "1".into(),
                    message: "OK".into(),
                    result: logs
                };

                return Ok(resp);
            }
            EventLogClient::EthScan(ref client) => {
                let resp = client.get_logs(params)
                    .await?;

                Ok(resp)
            }
        }
    }
}

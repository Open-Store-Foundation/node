use alloy::rpc::types::Log;
use serde::{Deserialize, Serialize};

pub type LogsResponse = EthScanResponse<Vec<Log>>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EthScanResponse<T> {
    pub status: String,
    pub message: String,
    pub result: T,
}

#[derive(Debug, Clone)]
pub struct GetLogsParams {
    pub from_block: u64,
    pub topic0: Option<String>,
    pub address: Option<String>,
    pub page: Option<u32>,
    pub offset: Option<u32>,
}
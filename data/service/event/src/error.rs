use thiserror::Error;

#[derive(Error, Debug)]
pub enum EventError {
    #[error("URL parsing error: {0}")]
    UrlParse(#[from] url::ParseError),

    #[error("HTTP client error: {0}")]
    HttpClient(String),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("RPC error: {0}")]
    Rpc(String),

    #[error("JSON deserialization error: {0}")]
    JsonDeserialization(#[from] serde_json::Error),

    #[error("Etherscan error: {0}")]
    Etherscan(#[from] client_ethscan::error::EthScanError),
}




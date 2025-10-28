use thiserror::Error;

#[derive(Error, Debug)]
pub enum EthScanError {
    #[error("URL parsing error: {0}")]
    UrlParse(#[from] url::ParseError),
    
    #[error("HTTP client error: {0}")]
    HttpClient(String),
    
    #[error("JSON deserialization error: {0}")]
    JsonDeserialization(#[from] serde_json::Error),
    
    #[error("API error: status={status}, message={message}")]
    ApiError { status: String, message: String },
    
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
}
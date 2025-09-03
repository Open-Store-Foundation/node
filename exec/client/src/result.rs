use alloy::transports::RpcError;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use db_redis::cache::KeyValueError;
use net_client::node::result::EthError;
use net_result::response_err;
use prost::DecodeError;
use serde_json::json;
use thiserror::Error;
use service_ethscan::error::EthScanError;

pub type ClientResult<T> = Result<T, ClientError>;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Database query failed: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("Migration failed: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("Item not found")]
    NotFound,

    #[error("Invalid input: {0}")]
    BadInput(String),

    #[error("JSON deserialization error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Error during working with cache: {0}")]
    CacheError(#[from] KeyValueError),

    #[error("Axus http error: {0}")]
    HttpError(#[from] axum::http::Error),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Eth error: {0}")]
    EthError(#[from] EthError),

    #[error("Eth scan error: {0}")]
    EthScanError(#[from] EthScanError),

    #[error("Decode error: {0}")]
    DecodeError(#[from] DecodeError),
}

pub enum ClientErrorCodes {
    Unknown = 0,
}

impl IntoResponse for ClientError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            ClientError::Sqlx(ref e) => {
                tracing::error!("Database error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR, ClientErrorCodes::Unknown,
                    "An internal error occurred".to_string(),
                )
            }
            ClientError::NotFound => (
                StatusCode::NOT_FOUND, ClientErrorCodes::Unknown,
                "Resource not found".to_string()
            ),
            ClientError::BadInput(msg) => (
                StatusCode::BAD_REQUEST, ClientErrorCodes::Unknown,
                msg
            ),
            ClientError::JsonError(ref e) => {
                tracing::error!("JSON error: {:?}", e);
                (
                    StatusCode::BAD_REQUEST, ClientErrorCodes::Unknown,
                    "Invalid JSON format".to_string()
                )
            }
            ClientError::Conflict(msg) => (
                StatusCode::CONFLICT, ClientErrorCodes::Unknown,
                msg
            ),
            ClientError::CacheError(msg) => (
                StatusCode::CONFLICT, ClientErrorCodes::Unknown,
                "An internal error occurred".to_string(),
            ),
            ClientError::Migration(e) => {
                tracing::error!("Migration error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR, ClientErrorCodes::Unknown,
                    "An internal error occurred".to_string(),
                )
            },
            ClientError::HttpError(e) => {
                tracing::error!("HTTP error: {:?}", e);

                (
                    StatusCode::INTERNAL_SERVER_ERROR, ClientErrorCodes::Unknown,
                    "An internal error occurred".to_string(),
                )
            },
            ClientError::EthScanError(e) => {
                tracing::error!("EthScan error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR, ClientErrorCodes::Unknown,
                    "An internal error occurred".to_string(),
                )
            },
            ClientError::EthError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR, ClientErrorCodes::Unknown,
                "An internal error occurred".to_string(),
            ),
            ClientError::DecodeError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR, ClientErrorCodes::Unknown,
                "An internal error occurred".to_string(),
            )
        };

        let error = response_err(code as i32, message );
        (status, error).into_response()
    }
}

use alloy::providers::PendingTransactionError;
use alloy::transports::{RpcError, TransportErrorKind};
use thiserror::Error;

pub type EthResult<T> = Result<T, EthError>;

#[derive(Debug, Error)]
pub enum EthError {
    #[error("Transaction failed")]
    TransactionFailed,
    #[error("Json eth error: {0}")]
    EthRpc(#[from] RpcError<TransportErrorKind>),
    #[error("Json eth error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Web3 eth error: {0}")]
    Web3Contract(#[from] alloy::contract::Error),
    #[error("Pending transaction eth error: {0}")]
    PendingTransactionError(#[from] PendingTransactionError),
}

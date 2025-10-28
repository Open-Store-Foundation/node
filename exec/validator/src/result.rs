use client_gf::client::GfError;
use net_client::node::result::EthError;
use prost::DecodeError;
use std::io;
use thiserror::Error;

pub type ValidatorResult<T> = Result<T, ValidatorError>;

#[derive(Error, Debug)]
pub enum ValidatorError {
    #[error("Io error: {0}")]
    Io(#[from] io::Error),

    #[error("Eth error: {0}")]
    Eth(#[from] EthError),

    #[error("Database query failed: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("Gf error: {0}")]
    Gf(#[from] GfError),

    #[error("Proto error: {0}")]
    Proto(#[from] DecodeError),
}

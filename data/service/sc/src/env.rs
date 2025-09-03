use std::env;
use std::env::VarError;

pub fn confirmations_env() -> Result<String, VarError> { env::var("CONFIRM_COUNT") }
pub fn confirmations() -> u64 {
    match confirmations_env() {
        Ok(data) => data.parse::<u64>()
            .expect("Failed to parse CONFIRM_COUNT"),
        _ => 0
    }
}

pub fn poll_timeout_ms_env() -> Result<String, VarError> { env::var("TX_POLL_TIMEOUT_MS") }
pub fn poll_timeout_ms() -> u64 {
    match poll_timeout_ms_env() {
        Ok(data) => data.parse::<u64>()
            .expect("Failed to parse TX_POLL_TIMEOUT_MS"),
        _ => 500
    }
}
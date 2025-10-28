use alloy::primitives::Address;
use std::env;
use std::env::VarError;
use std::str::FromStr;

// CONST
const VALIDATOR_VERSION: u64 = 1;
pub const fn validator_version() -> u64 { VALIDATOR_VERSION }

pub fn observe_timeout_sec() -> u64 { 30 }
pub fn poll_timeout_sec() -> u64 { 30 }

pub fn is_dev_env() -> bool { chain_id() == 31337 }
pub fn is_dev() -> bool { is_dev_env() }


// SERVICES
pub fn sqlite_path_env() -> Result<String, VarError> { env::var("DATABASE_URL") }
pub fn sqlite_path() -> String {
    sqlite_path_env()
        .expect("Can't find `DATABASE_URL` in .env")
}

pub fn eth_node_url_env() -> Result<String, VarError> { env::var("ETH_NODE_URL") }
pub fn eth_node_url() -> String {
    eth_node_url_env()
        .expect("Can't find `ETH_NODE_URL` in .env")   
}

pub fn gf_node_url_env() -> Result<String, VarError> { env::var("GF_NODE_URL") }
pub fn gf_node_url() -> String {
    gf_node_url_env()
        .expect("Can't find `GF_NODE_URL` in .env")  
}

pub fn file_storage_path_env() -> Result<String, VarError> { env::var("FILE_STORAGE_PATH") }
pub fn file_storage_path() -> String {
    file_storage_path_env()
        .expect("invalid FILE_STORAGE_PATH")
}

pub fn historical_sync_threshold_env() -> Result<String, VarError> { env::var("HISTORICAL_SYNC_THRESHOLD") }
pub fn historical_sync_threshold() -> u64 {
    historical_sync_threshold_env()
        .unwrap_or("500".to_string())
        .parse::<u64>()
        .expect("invalid historical sync threshold")
}


pub fn sync_retry_ms() -> u64 {
    return 60_000
}

pub fn sync_timeout_ms() -> u64 {
    return 60_000
}

pub fn max_logs_per_request() -> u32 {
    return 1_000
}

pub fn ethscan_api_key_env() -> Result<String, VarError> { env::var("ETHSCAN_API_KEY") }
pub fn ethscan_api_key() -> String {
    ethscan_api_key_env()
        .expect("Can't find `ETHSCAN_API_KEY` in .env")
}


// ADDRESSES
pub fn openstore_env() -> Result<String, VarError> { env::var("STORE_ADDRESS") }
pub fn openstore() -> String {
    openstore_env()
        .expect("Can't find `STORE_ADDRESS` in .env")
}
pub fn openstore_address() -> Address {
    Address::from_str(openstore().as_str())
        .expect("invalid openstore address")
}


// WALLET
pub fn chain_id_env() -> Result<String, VarError> { env::var("CHAIN_ID") }
pub fn chain_id() -> u64 {
    chain_id_env()
        .expect("Can't find `CHAIN_ID` in .env")
        .parse::<u64>()
        .expect("invalid chain id")
}

pub fn validator_pk_env() -> Result<String, VarError> { env::var("WALLET_PK") }
pub fn validator_pk() -> String {
    validator_pk_env()
        .expect("Can't find `WALLET_PK` in .env")
}


// TG
pub fn tg_token_env() -> Result<String, VarError> { env::var("TG_TOKEN") }
pub fn tg_token() -> String {
    tg_token_env()
        .expect("Can't find `TG_TOKEN` in .env")
}

pub fn info_chat_id_env() -> Result<String, VarError> { env::var("TG_INFO_CHAT_ID") }
pub fn info_chat_id() -> i64 {
    info_chat_id_env()
        .expect("Can't find `TG_INFO_CHAT_ID` in .env")
        .parse::<i64>()
        .expect("invalid chain id")
}

pub fn alert_chat_id_env() -> Result<String, VarError> { env::var("TG_ALERT_CHAT_ID") }
pub fn alert_chat_id() -> i64 {
    alert_chat_id_env()
        .expect("Can't find `TG_ALERT_CHAT_ID` in .env")
        .parse::<i64>()
        .expect("invalid chain id")
}
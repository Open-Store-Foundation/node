use std::env;
use std::env::VarError;
use std::str::FromStr;
use alloy::primitives::{Address};

// SERVICES
pub fn node_url_env() -> Result<String, VarError> { env::var("ETH_NODE_URL") }
pub fn node_url() -> String {
    node_url_env()
        .expect("Can't find `ETH_NODE_URL` in .env")
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

// ADDRESSES
pub fn assetlink_env() -> Result<String, VarError> { env::var("ORACLE_ADDRESS") }
pub fn assetlink() -> String {
    assetlink_env()
        .expect("Can't find `ORACLE_ADDRESS` in .env")   
}
pub fn assetlink_address() -> Address {
    Address::from_str(assetlink().as_str())
        .expect("invalid assetlink address")
}

// CONFIG
pub fn timeout_sec() -> u64 {
    return 60
}
pub fn timeout_empty_sec() -> u64 {
    return 60
}

// TG
pub fn tg_token_env() -> Result<String, VarError> { env::var("TG_TOKEN") }
pub fn tg_token() -> String {
    info_chat_id_env()
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
    chain_id_env()
        .expect("Can't find `TG_ALERT_CHAT_ID` in .env")
        .parse::<i64>()
        .expect("invalid chain id")
}

use alloy::primitives::Address;
use std::env;
use std::env::VarError;
use std::str::FromStr;

const GF_NODE_URL: &str = "GF_NODE_URL";
const GRAPH_NODE_URL: &str = "GRAPH_NODE_URL";
const ETH_NODE_URL: &str = "ETH_NODE_URL";
const ETHSCAN_API_KEY: &str = "ETHSCAN_API_KEY";

const CHAIN_ID: &str = "CHAIN_ID";
const WALLET_PK: &str = "WALLET_PK";

const HISTORICAL_SYNC_THRESHOLD: &str = "HISTORICAL_SYNC_THRESHOLD";
const HISTORICAL_SYNC_BLOCK: &str = "HISTORICAL_SYNC_BLOCK";

const ORACLE_ADDRESS: &str = "ORACLE_ADDRESS";
const STORE_ADDRESS: &str = "STORE_ADDRESS";

const CLIENT_HOST_URL: &str = "CLIENT_HOST_URL";
const REDIS_URL: &str = "REDIS_URL";
const DATABASE_URL: &str = "DATABASE_URL";

//////////////////////
// DAEMON
/////////////////////
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


// Nodes
pub fn eth_node_url_env() -> Result<String, VarError> { env::var(ETH_NODE_URL) }
pub fn eth_node_url() -> String {
    eth_node_url_env()
        .expect("Can't find `NODE_URL` in .env")
}

pub fn gf_node_url_env() -> Result<String, VarError> { env::var(GF_NODE_URL) }
pub fn gf_node_url() -> String {
    gf_node_url_env()
        .expect("Can't find `GF_NODE_URL` in .env")
}

pub fn graph_node_url_env() -> Result<String, VarError> { env::var(GRAPH_NODE_URL) }
pub fn graph_node_url() -> String {
    graph_node_url_env()
        .expect("Can't find `GRAPH_NODE_URL` in .env")
}

pub fn ethscan_api_key_env() -> Result<String, VarError> { env::var(ETHSCAN_API_KEY) }
pub fn ethscan_api_key() -> String {
    ethscan_api_key_env()
        .expect("Can't find `ETHSCAN_API_KEY` in .env")
}

// Wallet
pub fn chain_id_env() -> Result<String, VarError> { env::var(CHAIN_ID) }
pub fn chain_id() -> u64 {
    chain_id_env()
        .expect("Can't find `CHAIN_ID` in .env")
        .parse::<u64>()
        .expect("invalid chain id")
}

pub fn caip2() -> String {
    format!("eip155:{}", chain_id())
}

pub fn validator_pk_env() -> Result<String, VarError> { env::var(WALLET_PK) }
pub fn validator_pk() -> String {
    validator_pk_env()
        .expect("Can't find `WALLET_PK` in .env")
}

pub fn validator_version() -> u64 {
    return 0
}

// Sync
pub fn protocol_version() -> u64 { return 0 }

pub fn historical_sync_block_env() -> Result<String, VarError> { env::var(HISTORICAL_SYNC_BLOCK) }
pub fn historical_sync_block() -> u64 {
    return historical_sync_block_env()
        .unwrap_or("0".to_string())
        .parse::<u64>()
        .expect("invalid block number");
}

pub fn historical_sync_threshold_env() -> Result<String, VarError> { env::var(HISTORICAL_SYNC_THRESHOLD) }
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

// Addresses
pub fn assetlink_env() -> Result<String, VarError> { env::var(ORACLE_ADDRESS) }
pub fn assetlink() -> String {
    assetlink_env()
        .expect("Can't find `ORACLE_ADDRESS` in .env")
}
pub fn assetlink_address() -> Address {
    Address::from_str(assetlink().as_str())
        .expect("invalid assetlink address")
}

pub fn openstore_env() -> Result<String, VarError> { env::var(STORE_ADDRESS) }
pub fn openstore() -> String {
    openstore_env()
        .expect("Can't find `STORE_ADDRESS` in .env")
}
pub fn openstore_address() -> Address {
    Address::from_str(openstore().as_str())
        .expect("invalid openstore address")
}

//////////////////////
// API
/////////////////////
// Client
pub fn default_page_size() -> i64 { 20 }
pub fn client_host_url_env() -> Result<String, VarError> { env::var(CLIENT_HOST_URL) }
pub fn client_host_url() -> String {
    client_host_url_env()
        .unwrap_or("127.0.0.1:8081".to_string())
}

// Redis
pub fn redis_url_env() -> Result<String, VarError> {
    env::var(REDIS_URL)
}
pub fn redis_url() -> String {
    redis_url_env()
        .expect("Can't find `REDIS_URL` in .env!")   
}

// Psql
pub fn psql_url_env() -> Result<String, VarError> {
    env::var(DATABASE_URL)
}
pub fn psql_url() -> String {
    psql_url_env()
        .expect("Can't find `DATABASE_URL` in .env!")
}

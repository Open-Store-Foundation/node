use std::env;
use std::env::VarError;

// CONFIG
pub fn is_debug_env() -> bool { true }
pub fn is_debug() -> bool {
    is_debug_env()
}

pub fn stat_max_body_size() -> usize {
    1024 * 1024
}

// SERVICE
pub fn stat_host_url_env() -> Result<String, VarError> { env::var("STAT_HOST_URL") }
pub fn stat_host_url() -> String {
    stat_host_url_env()
        .unwrap_or("localhost:8082".into())
}

// CHICKHOUSE
pub fn ch_url_env() -> Result<String, VarError> { env::var("CLICKHOUSE_URL") }
pub fn ch_url() -> String {
    ch_url_env()
        .expect("Can't find ch url")
}

pub fn ch_user_env() -> Result<String, VarError> { env::var("CLICKHOUSE_USER") }
pub fn ch_user() -> String {
    ch_user_env()
        .expect("Can't find ch user")
}

pub fn ch_pass_env() -> Result<String, VarError> { env::var("CLICKHOUSE_PASSWORD") }
pub fn ch_pass() -> String {
    ch_pass_env()
        .expect("Can't find ch pass")
}

pub fn ch_db_env() -> Result<String, VarError> { env::var("CLICKHOUSE_DATABASE") }
pub fn ch_db() -> String {
    ch_db_env()
        .expect("Can't find ch db")
}

// KAFKA
pub fn kf_broker_env() -> Result<String, VarError> { env::var("KAFKA_BROKERS") }
pub fn kf_broker() -> String {
    kf_broker_env()
        .unwrap_or("127.0.0.1:9092".into())
}

pub fn kf_topic_env() -> Result<String, VarError> { env::var("KAFKA_TOPIC") }
pub fn kf_topic() -> String {
     kf_topic_env()
         .unwrap_or("stat-event".into())
}

pub fn kf_client_env() -> Result<String, VarError> { env::var("KAFKA_CLIENT") }
pub fn kf_client() -> String {
     kf_client_env()
         .unwrap_or("stat-default-client-1".into())
}

pub fn kf_group_env() -> Result<String, VarError> { env::var("KAFKA_GROUP") }
pub fn kf_group() -> String {
     kf_group_env()
         .unwrap_or("stat-default-group-1".into())
}

pub fn kf_key_env() -> Result<String, VarError> { env::var("KAFKA_KEY") }
pub fn kf_key() -> String {
    kf_topic_env().unwrap_or("stat-installation".into())
}

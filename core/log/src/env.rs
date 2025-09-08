use std::env;
use std::env::VarError;

pub fn log_path_env() -> Result<String, VarError> { env::var("LOG_PATH") }
pub fn log_path() -> String {
    log_path_env()
        .expect("Can't find `LOG_PATH` in .env")
}

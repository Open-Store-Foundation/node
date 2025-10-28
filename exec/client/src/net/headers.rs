use crate::env;
use crate::result::{ClientError, ClientResult};
use axum::http::HeaderMap;
use headers::HeaderValue;
use lazy_static::lazy_static;
use std::str::FromStr;

pub static API_VERSION: &'static str = "X-API-VERSION";

lazy_static! {
    static ref DEFAULT_API_VERSION: HeaderValue = HeaderValue::from(env::api_version());
}

pub trait ServiceHeaders {
    fn api_version(&self) -> ClientResult<u32>;
}

impl ServiceHeaders for HeaderMap {
    fn api_version(&self) -> ClientResult<u32> {
        let version = self.get(API_VERSION)
            .unwrap_or(&DEFAULT_API_VERSION);

        let Ok(str) = version.to_str() else {
            return Err(ClientError::BadInput("X-Api-Version format is incorrect!".into()));
        };

        let Ok(version) = u32::from_str(str) else {
            return Err(ClientError::BadInput("X-Api-Version format is incorrect!".into()));
        };

        return Ok(version);
    }
}

pub trait ApiNamedVersion {
    fn is_protocol_zero(&self) -> bool;
}

impl ApiNamedVersion for u32 {
    fn is_protocol_zero(&self) -> bool {
        return true;
    }
}

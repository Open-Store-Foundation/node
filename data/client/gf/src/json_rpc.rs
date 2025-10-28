// TODO to dedicated module

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcRequest {
    pub id: i64,
    pub jsonrpc: String,
    pub method: String,
    pub params: JsonRpcParams,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcParams {
    pub path: String,
    pub data: String,
    pub prove: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcResponse<T> {
    pub id: i64,
    pub jsonrpc: String,
    pub result: JsonRpcResult<T>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcResult<T> {
    pub response: T,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcResponseData {
    pub value: Option<String>,
}

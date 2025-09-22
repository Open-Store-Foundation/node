use alloy::rpc::types::Log;
use serde::{Deserialize, Deserializer, Serialize};
use serde::de::Error;
use serde_json::Value;

pub type LogsResponse = EthScanResponse;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EthScanResponse {
    pub status: String,
    pub message: String,
    #[serde(deserialize_with = "deserialize_vec_log_with_field_rename")]
    pub result: Vec<Log>,
}

#[derive(Debug, Clone)]
pub struct GetLogsParams {
    pub from_block: u64,
    pub topic0: Option<String>,
    pub address: Option<String>,
    pub page: Option<u32>,
    pub offset: Option<u32>,
}

fn deserialize_vec_log_with_field_rename<'de, D>(deserializer: D) -> Result<Vec<Log>, D::Error>
where
    D: Deserializer<'de>,
{
    // 1. Deserialize the incoming JSON into a generic serde_json::Value.
    // We expect it to be an array.
    let mut v: Value = Value::deserialize(deserializer)?;

    // 2. We expect the top-level value to be an array.
    // Iterate through each element and modify it.
    if let Some(logs_array) = v.as_array_mut() {
        for log_obj in logs_array {
            if let Some(obj_map) = log_obj.as_object_mut() {
                // Try to remove the "timestamp" field. If it exists...
                if let Some(timestamp_value) = obj_map.remove("timeStamp") {
                    // ...insert it back with the correct name, "block_timestamp".
                    obj_map.insert("blockTimestamp".to_string(), timestamp_value);
                }
            }
        }
    } else {
        return Err(Error::custom("Expected an array of logs"));
    }

    // 3. Deserialize from the now-modified Value into our concrete Vec<Log>.
    // This works because the field names in the Value now match the struct definition.
    Vec::<Log>::deserialize(v).map_err(Error::custom)
}


#[test]
fn test_parse_logs_response() {
    let resp: EthScanResponse = serde_json::from_str(
        r#"
        {
            "status": "1",
            "message": "OK",
            "result": [
                {
                    "address": "0x6edac88ea58168a47ab61836bcbad0ac844498a6",
                    "topics": [
                        "0xd6a0df45f78a8bd6872520058f9c76bc0d807517c9b1ccaa721e19826122982d"
                    ],
                    "data": "0x0000000000000000000000000000000000000000000000000000000000000002000000000000000000000000863536bf6b2c915174906b3fd48ae1c9855a2872",
                    "blockNumber": "0x3eac596",
                    "blockHash": "0x78e9541f1f4b5da8e1bd2046848c600a50f5400e3a9cb370e466a38bc447e2fa",
                    "timeStamp": "0x68caf7b6",
                    "gasPrice": "0x5f5e100",
                    "gasUsed": "0x18903",
                    "logIndex": "0x",
                    "transactionHash": "0x1da2ca86b107c2d467835d8ed57a0806605b672536ef38e283e30963946536b1",
                    "transactionIndex": "0x"
                }
            ]
        }
        "#
    ).unwrap();

    let time = resp.result.first().unwrap().block_timestamp;
    assert_ne!(time, None);
    println!("Timestamp {}", time.unwrap())
}
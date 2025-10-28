
pub mod client;
pub mod models;
pub mod error;

#[cfg(test)]
mod test {
    use alloy::rpc::types::Log;
    
    #[test]
    fn test_log_entry() {
        let example =
            r#"{
            "address": "0xf972f0477c4a5b054e00a19bbdf71f7941600bfc",
            "topics": [
                "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef",
                "0x0000000000000000000000005807e0dd3dd52e5a3bd7891822a14a150a47a9ea",
                "0x000000000000000000000000d3500220653b59cde1c2b856094bfbbc2dc1caaf"
            ],
            "data": "0x00000000000000000000000000000000000000000000003635c9adc5dea00000",
            "blockNumber": "0x39ea173",
            "blockHash": "0xa7d3f7a8199202aa281048a28a1a0535ef858d504783efebd0ecadafea1a6720",
            "timeStamp": "0x6891b926",
            "gasPrice": "0x5f5e100",
            "gasUsed": "0xd284",
            "logIndex": "0x1",
            "transactionHash": "0x754c9b86fb0fe20d219902d5451208f3a4d945a22190adbb97397585b740d7d9",
            "transactionIndex": "0x1"
        }"#;

        let log: Log = serde_json::from_str(example)
            .expect("failed to parse");

        println!("{:?}", log);
    }
}

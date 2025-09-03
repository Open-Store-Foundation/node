use std::time::Duration;
use alloy::network::Ethereum;
use alloy::providers::PendingTransactionBuilder;
use crate::env::confirmations;

pub trait ScDefaultCall {
    fn apply_call_settings(self) -> Self;
}

impl ScDefaultCall for PendingTransactionBuilder<Ethereum> {
    fn apply_call_settings(self) -> Self {
        return self
            .with_required_confirmations(confirmations())
            .with_timeout(Some(Duration::from_secs(120)))
    }
}

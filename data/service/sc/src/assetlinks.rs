use alloy::primitives::{Address, Log, B256, U256};
use alloy::rpc::types::TransactionReceipt;
use alloy::sol_types::SolEvent;
use codegen_contracts::contracts::AssetlinksOracle;
use derive_more::{Display, From};
use net_client::node::provider::Web3Provider;
use net_client::node::result::EthResult;
use net_client::node::watcher::TxWorkaround;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::env;

#[derive(Debug, Eq, Display, From, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum AssetlinkStatusCode {
    Undefined = 0,
    Success = 1,
    ExceedRpcAttemptsErrors = 2,
    UrlFormatError = 3,
    WebsiteFormatError = 4,
    UnreachableLinkError = 5,
    AssetlinkFormatError = 6,
    ContentReadingError = 7,
    NoPackageError = 8,
    NoFingerprintError = 9,
}

impl From<i32> for AssetlinkStatusCode {
    fn from(value: i32) -> Self {
        match value {
            1 => AssetlinkStatusCode::Success,
            2 => AssetlinkStatusCode::ExceedRpcAttemptsErrors,
            3 => AssetlinkStatusCode::UrlFormatError,
            4 => AssetlinkStatusCode::WebsiteFormatError,
            5 => AssetlinkStatusCode::UnreachableLinkError,
            6 => AssetlinkStatusCode::AssetlinkFormatError,
            7 => AssetlinkStatusCode::ContentReadingError,
            8 => AssetlinkStatusCode::NoPackageError,
            9 => AssetlinkStatusCode::NoFingerprintError,
            _ => AssetlinkStatusCode::Undefined,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct VerificationState {
    pub status: AssetlinkStatusCode,
}

pub struct ScAssetLinkService {
    assetlink: Address,
    provider: Arc<Web3Provider>,
}

impl ScAssetLinkService {
    pub fn new(assetlink: Address, client: &Arc<Web3Provider>) -> Self {
        Self { provider: client.clone(), assetlink }
    }
}

impl ScAssetLinkService {

    pub const ENQUEUE_VERIFICATION_HASH: B256 = AssetlinksOracle::EnqueueVerification::SIGNATURE_HASH;
    pub const SYNC_FINISH_HASH: B256 = AssetlinksOracle::FinalizeVerification::SIGNATURE_HASH;

    pub fn decode_enqueue_log(data: &[u8]) -> alloy::sol_types::Result<(Address, u64, i64)> {
        let result = AssetlinksOracle::EnqueueVerification::abi_decode_data(data)?;
        return Ok((result.0, result.1.to(), result.2.to()));
    }

    pub fn decode_finalize_log(data: &Log) -> alloy::sol_types::Result<Log<AssetlinksOracle::FinalizeVerification>> {
        let result = AssetlinksOracle::FinalizeVerification::decode_log(data);
        return result;
    }

    pub async fn get_app_from_queue(&self, request_id: i64) -> EthResult<(Address, u64)> {
        let contract = AssetlinksOracle::new(self.assetlink, &self.provider);

        let result = contract.queue(U256::from(request_id))
            .call()
            .await?;

        return Ok((result.target, result.version.to()));
    }

    pub async fn get_state(&self) -> EthResult<(i64, i64, u64)> {
        let contract = AssetlinksOracle::new(self.assetlink, &self.provider);

        let result = contract.getContractState()
            .call()
            .await?;

        return Ok((result._0.to(), result._1.to(), result._2.to()));
    }

    pub async fn finish(&self, request_id: i64, state: &VerificationState) -> EthResult<TransactionReceipt> {
        let contract = AssetlinksOracle::new(self.assetlink, &self.provider);

        let request = contract.finish(U256::from(request_id), U256::from(state.status as i32))
            .into_transaction_request();

        return self.provider.send_and_wait(request, env::poll_timeout_ms())
            .await
    }
}

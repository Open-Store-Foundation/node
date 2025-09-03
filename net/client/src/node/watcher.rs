use crate::node::provider::Web3Provider;
use crate::node::result::{EthError, EthResult};
use alloy::eips::Encodable2718;
use alloy::network::Ethereum;
use alloy::primitives::B256;
use alloy::providers::{PendingTransactionBuilder, Provider, SendableTx};
use alloy::rpc::types::{TransactionReceipt, TransactionRequest};
use alloy::transports::TransportResult;
use tokio::time::sleep;

pub trait TxWorkaround {
    async fn send_tx(&self, request: TransactionRequest) -> TransportResult<PendingTransactionBuilder<Ethereum>>;
    async fn wait_for_tx(&self, tx_hash: B256, timeout: u64) -> EthResult<TransactionReceipt>;
    async fn send_and_wait(&self,  request: TransactionRequest, timeout: u64) -> EthResult<TransactionReceipt>;
}

impl TxWorkaround for Web3Provider {
    
    async fn send_tx(
        &self,
        request: TransactionRequest,
    ) -> TransportResult<PendingTransactionBuilder<Ethereum>> {
        let tx = self.fill(request).await?;

        // if let Some(builder) = tx.as_builder() {
        //     if let FillerControlFlow::Missing(missing) = self.filler.status(builder) {
        //         // TODO: improve this.
        //         // blocked by #431
        //         let message = format!("missing properties: {missing:?}");
        //         return Err(RpcError::local_usage_str(&message));
        //     }
        // }

        match tx {
            SendableTx::Builder(mut tx) => {
                alloy_network::TransactionBuilder::prep_for_submission(&mut tx);
                let tx_hash = self.client().request("eth_sendTransaction", (tx,)).await?;
                Ok(PendingTransactionBuilder::new(self.root().clone(), tx_hash))
            }
            SendableTx::Envelope(tx) => {
                let encoded_tx = tx.encoded_2718();
                self.send_raw_transaction(&encoded_tx).await
            }
        }
    }

    async fn wait_for_tx(&self, tx_hash: B256, timeout: u64) -> EthResult<TransactionReceipt> {
        let mut receipt: Option<TransactionReceipt> = None;

        loop {
            let result = self.get_transaction_receipt(tx_hash).await?;
            if let Some(data) = result {
                receipt = Some(data);
                break;
            } else {
                sleep(std::time::Duration::from_millis(timeout)).await;
            }
        }

        let check = receipt.expect("Transaction can't be None");

        if !check.status() {
            return Err(EthError::TransactionFailed)
        }

        return Ok(check)
    }

    async fn send_and_wait(&self, request: TransactionRequest, timeout: u64) -> EthResult<TransactionReceipt> {
        let result = self.send_tx(request)
            .await?;

        return self.wait_for_tx(result.tx_hash().clone(), timeout)
            .await
    }
}


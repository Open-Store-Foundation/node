use alloy::eips::eip1559::Eip1559Estimation;
use alloy::providers::fillers::{FillerControlFlow, TxFiller};
use alloy::providers::{Provider, SendableTx};
use alloy::transports::TransportResult;
use alloy_network::{Network, TransactionBuilder};
use futures_util::FutureExt;
use std::future::IntoFuture;

#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GasFillable {
    Eip1559 { gas_limit: u64, estimate: Eip1559Estimation },
}

#[derive(Clone, Copy, Debug, Default)]
pub struct MxGasFiller {
    mx_limit: f64,
    mx_price: f64,
}

impl MxGasFiller {
    pub fn new(mx_limit: f64, mx_price: f64) -> Self {
        Self { mx_limit, mx_price }
    }
}

impl MxGasFiller {
    async fn prepare_1559<P, N>(
        &self,
        provider: &P,
        tx: &N::TransactionRequest,
    ) -> TransportResult<GasFillable>
    where
        P: Provider<N>,
        N: Network,
    {
        let gas_limit_fut = tx.gas_limit().map_or_else(
            || provider.estimate_gas(tx.clone()).into_future().right_future(),
            |gas_limit| async move { Ok(gas_limit) }.left_future(),
        );

        let eip1559_fees_fut = if let (Some(max_fee_per_gas), Some(max_priority_fee_per_gas)) =
            (tx.max_fee_per_gas(), tx.max_priority_fee_per_gas())
        {
            async move { Ok(Eip1559Estimation { max_fee_per_gas, max_priority_fee_per_gas }) }
                .left_future()
        } else {
            provider.estimate_eip1559_fees()
                .right_future()
        };

        let (gas_limit, estimate) = futures::try_join!(gas_limit_fut, eip1559_fees_fut)?;

        Ok(GasFillable::Eip1559 { gas_limit, estimate })
    }
}

impl <N: Network> TxFiller<N> for MxGasFiller {
    type Fillable = GasFillable;

    fn status(&self, tx: &<N as Network>::TransactionRequest) -> FillerControlFlow {
        // legacy and eip2930 tx
        if tx.gas_price().is_some() && tx.gas_limit().is_some() {
            return FillerControlFlow::Finished;
        }

        // eip1559
        if tx.max_fee_per_gas().is_some()
            && tx.max_priority_fee_per_gas().is_some()
            && tx.gas_limit().is_some()
        {
            return FillerControlFlow::Finished;
        }

        FillerControlFlow::Ready
    }

    fn fill_sync(&self, _tx: &mut SendableTx<N>) {}

    async fn prepare<P>(
        &self,
        provider: &P,
        tx: &<N as Network>::TransactionRequest,
    ) -> TransportResult<Self::Fillable>
    where
        P: Provider<N>,
    {
        match self.prepare_1559(provider, tx).await {
            // fallback to legacy
            Ok(estimate) => Ok(estimate),
            Err(e) => Err(e),
        }
    }

    async fn fill(
        &self,
        fillable: Self::Fillable,
        mut tx: SendableTx<N>,
    ) -> TransportResult<SendableTx<N>> {
        if let Some(builder) = tx.as_mut_builder() {
            match fillable {
                GasFillable::Eip1559 { gas_limit, estimate } => {
                    let mx_gas_limit = (self.mx_limit * (gas_limit as f64)) as u64;
                    let mx_gas_price = (self.mx_price * (estimate.max_fee_per_gas as f64)) as u128;
                    let mx_max_priority_fee_per_gas = (self.mx_price * (estimate.max_priority_fee_per_gas as f64)) as u128;
                    
                    builder.set_gas_limit(mx_gas_limit);
                    builder.set_max_fee_per_gas(mx_gas_price);
                    builder.set_max_priority_fee_per_gas(mx_max_priority_fee_per_gas);
                }
            }
        };
        Ok(tx)
    }
}

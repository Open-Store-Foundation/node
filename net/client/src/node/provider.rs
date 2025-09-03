use crate::node::service::gas::MxGasFiller;
use alloy::network::EthereumWallet;
use alloy::providers::fillers::{ChainIdFiller, FillProvider, JoinFill, NonceFiller, SimpleNonceManager, WalletFiller};
use alloy::providers::{Identity, ProviderBuilder, RootProvider};
use alloy::rpc::client::RpcClient;
use alloy::transports::http::Http;
use alloy::transports::layers::{RetryBackoffLayer, RetryBackoffService};
use reqwest::{Client, Url};
use std::time::Duration;
use tower::ServiceBuilder;

pub struct Web3ProviderFactory;

pub type HttpProvider = RetryBackoffService<Http<Client>>;
pub type Web3Provider = FillProvider<JoinFill<JoinFill<JoinFill<JoinFill<Identity, NonceFiller<SimpleNonceManager>>, MxGasFiller>, ChainIdFiller>, WalletFiller<EthereumWallet>>, RootProvider<>>;

impl Web3ProviderFactory {

    pub fn http(rpc_url: Url, client: &Client) -> HttpProvider {
        let http = Http::with_client(client.clone(), rpc_url);

        let retry_layer = RetryBackoffLayer::new(
            3, 3_000, 500 // TODO 500 to env
        );

        let service = ServiceBuilder::new()
            .layer(retry_layer)
            // TODO return in case for problems .layer(LoggingLayer)
            .service(http);

        return service;
    }

    pub fn provider(
        rpc_url: Url, 
        chain_id: u64, 
        client: &Client, 
        wallet: EthereumWallet
    ) -> Web3Provider {
        let is_local = false; // guess_local_url(&node);
        let http = Self::http(rpc_url, client);

        let layer_transport = RpcClient::builder()
            .transport(http, is_local)
            .with_poll_interval(Duration::from_secs(60));

        layer_transport.inner()
            .set_poll_interval(Duration::from_secs(60));

        let provider = ProviderBuilder::default()
            .filler(NonceFiller::new(SimpleNonceManager::default()))
            .filler(MxGasFiller::new(1.0, 1.1))
            .filler(ChainIdFiller::new(Some(chain_id)))
            .wallet(wallet)
            .connect_client(layer_transport);

        return provider;
    }
}


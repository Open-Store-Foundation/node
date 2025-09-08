use crate::launcher::{OracleEvent, OracleHandler, OracleQueue};
use crate::verifier::android::app_verifier::AndroidAppVerifier;
use crate::verifier::asset_provider::AssetProvider;
use alloy::providers::Provider;
use alloy::sol_types::SolType;
use base64::Engine;
use client_tg::client::TgClientSettings;
use core_log::init_tracer;
use core_std::arc;
use core_std::profile::is_debug;
use dotenvy::dotenv;
use net_client::http::HttpProviderFactory;
use net_client::node::provider::{HttpProvider, Web3Provider, Web3ProviderFactory};
use net_client::node::signer::ValidatorSigner;
use serde::{Deserialize, Serialize};
use service_sc::assetlinks::ScAssetLinkService;
use service_sc::obj::ScObjService;
use service_sc::store::ScStoreService;
use std::convert::Into;
use std::str::FromStr;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::signal;
use tokio::sync::mpsc;
use tokio::time::sleep;
use tracing::warn;
use core_std::shutdown::shutdown_signal;

mod launcher;
mod result;
mod verifier;
mod env;

// http://localhost:1984/mint/MlV6DeOtRmakDOf6vgOBlif795tcWimgyPsYYNQ8q1Y/100000000000000000000
#[tokio::main]
async fn main() {
    dotenv().ok();
    let _guard = init_tracer();
    
    if !is_debug() {
        let settings = TgClientSettings {
            token: env::tg_token(),
            client_name: "ORALE".into(),
            msg_chat_id: env::info_chat_id(),
            alert_chat_id: env::alert_chat_id(),
        };
        
        client_tg::init_with_client(
            settings,
            HttpProviderFactory::http_client()
                .expect("Failed to create http client"),
        );
    }
    
    let url = env::node_url();

    let client = HttpProviderFactory::http_client()
        .expect("Failed to create http client");
    let pk = arc!(
        ValidatorSigner::new(env::validator_pk())
        .expect("PrivateKey hex is not valid!")
    );
    let rpc_url = url.parse()
        .expect("Failed to parse rpc_node_url");
    let web3 = arc!(Web3ProviderFactory::provider(rpc_url, env::chain_id(), &client, pk.wallet()));

    let asset_provider = arc!(ScAssetLinkService::new(env::assetlink_address(), &web3));
    let app_provider = arc!(ScObjService::new(web3.clone()));

    let assets = arc!(AssetProvider::new(&client));
    let app_verifier = arc!(AndroidAppVerifier::new(
        &assets,
        &asset_provider,
        &app_provider
    ));
    let handler = arc!(OracleHandler::new(
        env::timeout_sec(),
        env::timeout_empty_sec(),
        &web3,
        &asset_provider,
        &app_provider,
        &assets,
        &app_verifier
    ));

    let queue = arc!(OracleQueue::new(100));
    let e_oracle = queue.clone();
    tokio::spawn(async move {
        tokio::select! {
            _ = shutdown_signal() => {
                warn!("Shutdown oracle queue!");
                e_oracle.async_shutdown()
                    .await;
            },
        }
    });

    let oracle_c = queue.clone();
    let handler_c = handler.clone();

    let oracle_task = tokio::spawn(async move {
        oracle_c.push(OracleEvent::Poll)
            .await;

        let _ = oracle_c.run(handler_c)
            .await;
    });

    tokio::join!(oracle_task);
    while !queue.is_shutdown_finished() {
        warn!("Waiting for parallel tasks shutdown signal");
        sleep(Duration::from_secs(5)).await;
    }
}

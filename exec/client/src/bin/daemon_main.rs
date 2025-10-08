use alloy::sol_types::sol_data::String;
use client::daemon::data::obj_info_provider::DaemonFactory;
use client::daemon::handler::chain_sync::ChainSyncHandler;
use client::daemon::handler::sync::add_to_track::AddToTrack;
use client::daemon::handler::sync::block_finalized::BlockFinalizedHandler;
use client::daemon::handler::sync::new_req::NewRequestHandler;
use client::daemon::handler::sync::sync_finish::SyncFinishedHandler;
use client::daemon::launcher::{DaemonAction, DaemonEventHandler, DaemonQueue};
use client::data::repo::artifact_repo::ArtifactRepo;
use client::data::repo::assetlink_repo::AssetlinkRepo;
use client::data::repo::batch_repo::BatchRepo;
use client::data::repo::error_repo::ErrorRepo;
use client::data::repo::object_repo::ObjectRepo;
use client::data::repo::publishing_repo::PublishingRepo;
use client::data::repo::validation_repo::ValidationRepo;
use client::env;
use client::env::{psql_url};
use client_tg::client::{TgClient, TgClientSettings};
use client_tg::tg_alert;
use cloud_gf::client::GreenfieldClient;
use codegen_block::block::{ValidationBlock, ValidationProofs, ValidationResult};
use core_std::arc;
use core_std::profile::is_debug;
use db_psql::client::PgClient;
use dotenvy::dotenv;
use lazy_static::lazy_static;
use net_client::http::HttpProviderFactory;
use net_client::node::provider::Web3ProviderFactory;
use net_client::node::signer::ValidatorSigner;
use prost::Message;
use service_ethscan::client::EthScanClient;
use service_graph::client::GraphClient;
use service_ethscan::models::GetLogsParams;
use service_sc::obj::ScObjService;
use service_sc::store::ScStoreService;
use std::time::Duration;
use tokio::signal;
use tokio::time::sleep;
use tracing::{error, info, warn};
use core_std::shutdown::shutdown_signal;

#[tokio::main]
async fn main() {
    info!("Starting daemon!");
    dotenv().ok();
    let _guard = core_log::init_tracer();

    if !is_debug() {
        let settings = TgClientSettings {
            token: env::tg_token(),
            client_name: "DAEMON".into(),
            msg_chat_id: env::info_chat_id(),
            alert_chat_id: env::alert_chat_id(),
        };

        client_tg::init_with_client(
            settings,
            HttpProviderFactory::http_client()
                .expect("Failed to create http client"),
        );
    }

    info!("Connecting to database...");
    let pg_client = PgClient::connect(psql_url().as_ref())
        .await;

    let pg_client = match pg_client {
        Ok(c) => c,
        Err(e) => {
            error!("Database connection error: {}", e);
            return;
        }
    };
    info!("Database connected successfully.");
    
    let publishing_repo =  arc!(PublishingRepo::new(pg_client.clone()));
    let assetlink_repo = arc!(AssetlinkRepo::new(pg_client.clone()));
    let artifact_repo = arc!(ArtifactRepo::new(pg_client.clone()));
    let object_repo = arc!(ObjectRepo::new(pg_client.clone()));
    let validation_repo = arc!(ValidationRepo::new(pg_client.clone()));
    let error_repo = arc!(ErrorRepo::new(pg_client.clone()));
    let batch_repo = arc!(BatchRepo::new(pg_client.clone()));

    let pk = arc!(ValidatorSigner::new(env::validator_pk()).expect("PrivateKey hex is not valid!"));
    let url = env::eth_node_url();

    // TODO use for requests cache(especially greenfield)
    // let cache = Cache::<String, String>::builder()
    //     .time_to_live(Duration::from_secs(10 * 60))
    //     .max_capacity(1000)
    //     .build();
    
    let client = HttpProviderFactory::http_client()
        .expect("Failed to create http client");
    let rpc_url = url.parse()
        .expect("Failed to parse rpc_node_url");
    let web3 = arc!(Web3ProviderFactory::provider(rpc_url, env::chain_id(), &client, pk.wallet()));

    let greenfield = arc!(GreenfieldClient::new(client.clone(), env::gf_node_url(), Some(pk.clone())));
    let graph = arc!(GraphClient::new(client.clone(), env::graph_node_url().parse().expect("Invalid GRAPH_NODE_URL")));
    let store_service = arc!(ScStoreService::new(env::openstore_address(), env::validator_version(), &web3));
    let obj_service = arc!(ScObjService::new(web3.clone()));
    let ethscan_client = arc!(EthScanClient::new(client.clone(), env::chain_id(), env::ethscan_api_key()));

    let factory = arc!(
        DaemonFactory::new(obj_service.clone(), greenfield.clone())
    );

    let block_finalized_handler = arc!(
        BlockFinalizedHandler::new(
            factory.clone(),
            store_service.clone(),
            object_repo.clone(),
            publishing_repo.clone(),
            artifact_repo.clone(),
            validation_repo.clone(),
            error_repo.clone(),
        )
    );

    let req_new_handler = arc!(
        NewRequestHandler::new(
            factory.clone(),
            object_repo.clone(),
            artifact_repo.clone(),
            validation_repo.clone(),
            error_repo.clone(),
        )
    );

    let sync_finish_handler = arc!(
        SyncFinishedHandler::new(
            factory.clone(),
            obj_service.clone(),
            object_repo.clone(),
            assetlink_repo.clone(),
            error_repo.clone(),
        )
    );

    let add_to_track_handler = arc!(
        AddToTrack::new(
            factory.clone(),
            object_repo.clone(),
            publishing_repo.clone(),
            error_repo.clone(),
        )
    );

    let sync = arc!(
        ChainSyncHandler::new(
            env::historical_sync_block(),
            env::historical_sync_threshold(),
            Duration::from_millis(env::sync_retry_ms()),
            Duration::from_millis(env::sync_timeout_ms()),
            web3.clone(),
            ethscan_client.clone(),
            graph.clone(),
            object_repo.clone(),
            batch_repo.clone(),
            sync_finish_handler.clone(),
            req_new_handler.clone(),
            block_finalized_handler.clone(),
            add_to_track_handler.clone(),
        )
    );

    let daemon = arc!(DaemonEventHandler::new(sync.clone()));
    let queue = arc!(DaemonQueue::new(100));

    info!("Demon deps created.");
    
    let e_queue = queue.clone();
    tokio::spawn(async move {
        tokio::select! {
            () = shutdown_signal() => {
                warn!("Shutdown validator queue");
                e_queue.push_sequential(DaemonAction::Shutdown)
                    .await;
            },
        }
    });

    let queue_c = queue.clone();
    let daemon_c = daemon.clone();
    let daemon_task = tokio::spawn(async move {
        queue_c.push(DaemonAction::Launch)
            .await;

        let _ = queue_c.run(daemon_c)
            .await;
    });

    let _ = tokio::join!(daemon_task);

    while !queue.is_shutdown_finished() {
        warn!("Waiting for parallel tasks shutdown signal");
        sleep(Duration::from_secs(5)).await;
    }
}

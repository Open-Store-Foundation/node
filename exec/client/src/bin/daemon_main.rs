use alloy::sol_types::sol_data::String;
use alloy::transports::http::reqwest::Url;
use client::daemon::data::data_sync::DataSyncHandler;
use client::daemon::data::object_factory::ObjectFactory;
use client::daemon::handler::chain_sync_v0::ChainSyncHandlerV0;
use client::daemon::handler::chain_sync_v1::ChainSyncHandlerV1;
use client::daemon::handler::sync::add_to_track::AddToTrack;
use client::daemon::handler::sync::block_finalized::BlockFinalizedHandler;
use client::daemon::handler::sync::new_req::NewRequestHandler;
use client::daemon::handler::sync::new_req_v0::NewRequestHandlerV0;
use client::daemon::handler::sync::sync_finish::SyncFinishedHandler;
use client::daemon::handler::sync::sync_finish_v0::SyncFinishedHandlerV0;
use client::daemon::launcher::{DaemonAction, DaemonEventHandler, DaemonQueue};
use client::data::repo::artifact_repo::ArtifactRepo;
use client::data::repo::assetlink_repo::AssetlinkRepo;
use client::data::repo::batch_repo::BatchRepo;
use client::data::repo::error_repo::ErrorRepo;
use client::data::repo::object_repo::ObjectRepo;
use client::data::repo::publishing_repo::PublishingRepo;
use client::data::repo::validation_repo::ValidationRepo;
use client::env;
use client::env::psql_url;
use client::util::proof_validator::ProofValidator;
use client_tg::client::{TgClient, TgClientSettings};
use client_tg::tg_alert;
use client_gf::client::GreenfieldClient;
use codegen_block::block::{ValidationBlock, ValidationProofs, ValidationResult};
use core_std::arc;
use core_std::profile::is_debug;
use core_std::shutdown::shutdown_signal;
use core_std::url::Localhost;
use db_psql::client::PgClient;
use dotenvy::dotenv;
use lazy_static::lazy_static;
use net_client::http::HttpProviderFactory;
use net_client::node::provider::Web3ProviderFactory;
use net_client::node::signer::ValidatorSigner;
use prost::Message;
use client_ethscan::client::EthScanClient;
use client_ethscan::models::GetLogsParams;
use service_event::service::{EventLogService, EventLogClient};
use service_graph::client::GraphClient;
use service_sc::obj::ScObjService;
use service_sc::store::ScStoreService;
use std::time::Duration;
use tokio::signal;
use tokio::time::sleep;
use tracing::{error, info, warn};

// TODO use for requests cache(especially greenfield)
// let cache = Cache::<String, String>::builder()
//     .time_to_live(Duration::from_secs(10 * 60))
//     .max_capacity(1000)
//     .build();
#[tokio::main]
async fn main() {
    info!("Starting daemon!");
    dotenv().ok();
    let _guard = core_log::init_tracer();

    if !is_debug() {
        client_tg::init_with_client(
            TgClientSettings {
                token: env::tg_token(),
                client_name: "DAEMON".into(),
                msg_chat_id: env::info_chat_id(),
                alert_chat_id: env::alert_chat_id(),
            },

            HttpProviderFactory::http_client()
                .expect("Failed to create http client"),
        );
    }

    info!("Connecting to database...");
    let pg_client = PgClient::connect(psql_url().as_ref()).await;
    let pg_client = match pg_client {
        Ok(c) => c,
        Err(e) => {
            error!("Database connection error: {}", e);
            return;
        }
    };
    info!("Database connected successfully.");


    info!("Create DB repositories...");
    let publishing_repo = arc!(PublishingRepo::new(pg_client.clone()));
    let assetlink_repo = arc!(AssetlinkRepo::new(pg_client.clone()));
    let artifact_repo = arc!(ArtifactRepo::new(pg_client.clone()));
    let object_repo = arc!(ObjectRepo::new(pg_client.clone()));
    let validation_repo = arc!(ValidationRepo::new(pg_client.clone()));
    let error_repo = arc!(ErrorRepo::new(pg_client.clone()));
    let batch_repo = arc!(BatchRepo::new(pg_client.clone()));

    info!("Create Web3 providers...");
    let node_url = env::eth_node_url().parse::<Url>()
        .expect("Failed to parse rpc_node_url");
    let pk = arc!(ValidatorSigner::new(env::validator_pk())
        .expect("PrivateKey hex is not valid!"));
    let client = HttpProviderFactory::http_client().
        expect("Failed to create http client");
    let web3 = arc!(Web3ProviderFactory::provider(
        node_url.clone(),
        env::chain_id(),
        &client,
        pk.wallet()
    ));
    let greenfield = arc!(GreenfieldClient::new(
        client.clone(),
        env::gf_node_url(),
        None
    ));
    let graph = arc!(GraphClient::new(
        client.clone(),
        env::graph_node_url()
            .parse()
            .expect("Invalid GRAPH_NODE_URL")
    ));

    let store_service = arc!(ScStoreService::new(
        env::openstore_address(),
        env::validator_version(),
        &web3
    ));
    let obj_service = arc!(ScObjService::new(web3.clone()));
    let ethscan_client = arc!(EthScanClient::new(
        client.clone(),
        env::chain_id(),
        env::ethscan_api_key()
    ));

    let event_service = if node_url.is_localhost() {
        arc!(EventLogService::new(EventLogClient::Eth(web3.clone())))
    } else {
        arc!(EventLogService::new(EventLogClient::EthScan(ethscan_client.clone())))
    };

    info!("Create handlers...");
    let factory = arc!(ObjectFactory::new(
        obj_service.clone(),
        greenfield.clone()
    ));
    let proof_verifier = arc!(ProofValidator::new(
        env::caip2(), 
        env::protocol_version(),
        obj_service.clone(),
    ));

    let sync_finish_handler = arc!(SyncFinishedHandlerV0::new(
        factory.clone(),
        proof_verifier.clone(),
        obj_service.clone(),
        object_repo.clone(),
        assetlink_repo.clone(),
        error_repo.clone(),
    ));

    let req_new_handler = arc!(NewRequestHandlerV0::new(
        factory.clone(),
        object_repo.clone(),
        error_repo.clone(),
    ));

    let data_sync_handler = arc!(DataSyncHandler::new(
        pg_client.clone(),
        object_repo.clone(),
        batch_repo.clone(),
        assetlink_repo.clone(),
        artifact_repo.clone(),
        validation_repo.clone(),
        publishing_repo.clone(),
        error_repo.clone(),
    ));

    let sync = arc!(ChainSyncHandlerV0::new(
        env::historical_sync_block(),
        Duration::from_millis(env::sync_retry_ms()),
        Duration::from_millis(env::sync_timeout_ms()),
        web3.clone(),
        event_service.clone(),
        graph.clone(),
        data_sync_handler.clone(),
        sync_finish_handler.clone(),
        req_new_handler.clone(),
    ));

    info!("Launch daemon...");
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
        queue_c.push(DaemonAction::Launch).await;

        let _ = queue_c.run(daemon_c).await;
    });

    let _ = tokio::join!(daemon_task);

    while !queue.is_shutdown_finished() {
        warn!("Waiting for parallel tasks shutdown signal");
        sleep(Duration::from_secs(5)).await;
    }
}

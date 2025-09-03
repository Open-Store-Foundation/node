use crate::android::apk::chunker::ApkChunker;
use crate::android::apk::parser::ApkParser;
use crate::android::apk::verifier::ApkVerifierV2;
use crate::android::build::AndroidBuildVerifier;
use crate::android::validator::AndroidValidator;
use crate::data::validation_repo::ValidationRepo;
use crate::handlers::check_proposal::CheckProposalHandler;
use crate::handlers::common::create_proposal::CreateProposalCase;
use crate::handlers::common::top_up::TopUpCase;
use crate::handlers::common::validation::ValidationCase;
use crate::handlers::finalize::FinalizeHandler;
use crate::handlers::observe_overdue::ObserveOverdueHandler;
use crate::handlers::observe_voting::ObserveVotingHandler;
use crate::handlers::poll::{PollHandler, ValidatorEventPoolConfig};
use crate::handlers::propose::ProposeHandler;
use crate::handlers::register::RegisterHandler;
use crate::handlers::sync::SyncHandler;
use crate::handlers::try_assign::TryAssignHandler;
use crate::handlers::unregister::UnregisterHandler;
use crate::handlers::validate_sync::ValidateSyncHandler;
use crate::handlers::vote::VoteHandler;
use crate::launcher::{ValidationQueue, ValidatorEvent, ValidatorQueue};
use alloy::providers::Provider;
use client_tg::client::TgClientSettings;
use cloud_gf::client::GreenfieldClient;
use codegen_contracts::ext::ToChecksum;
use core_log::init_tracer;
use core_std::arc;
use core_std::profile::is_debug;
use data::block_repo::BlockRepo;
use data::file_storage::FileStorage;
use db_sqlite::client::SqliteClient;
use dotenvy::dotenv;
use net_client::http::HttpProviderFactory;
use net_client::node::provider::Web3ProviderFactory;
use net_client::node::signer::ValidatorSigner;
use service_sc::obj::ScObjService;
use service_sc::store::ScStoreService;
use std::time::Duration;
use tokio::signal;
use tokio::time::sleep;
use tracing::{info, warn};

mod android;
mod env;
mod handlers;
mod launcher;
mod data;
mod result;
mod utils;
mod ext;

#[tokio::test]
async fn test() {
    info!("Starting validator");
    dotenv().ok();
    init_tracer();

    let pk = arc!(ValidatorSigner::new(env::validator_pk()).expect("PrivateKey hex is not valid!"));
    let url = env::eth_node_url();
    let version = env::validator_version();
    
    let client = HttpProviderFactory::http_client()
        .expect("Failed to create http client");
    let rpc_url = url.parse()
        .expect("Failed to parse rpc_node_url");
    
    let web3 = arc!(Web3ProviderFactory::provider(rpc_url, env::chain_id(), &client, pk.wallet()));
    let store_service = arc!(ScStoreService::new(env::openstore_address(), version, &web3));

    // let res = web3.estimate_eip1559_fees().await;
    // println!("{:?}", res);
    // let result = store_service.unregister_validator().await;
    // println!("{:?}", result);
}

// http://localhost:1984/mint/MlV6DeOtRmakDOf6vgOBlif795tcWimgyPsYYNQ8q1Y/100000000000000000000
#[tokio::main]
async fn main() {
    dotenv().ok();
    init_tracer();
    info!("Starting validator");

    if !is_debug() {
        info!("Start TG");
        let settings = TgClientSettings {
            token: env::tg_token(),
            client_name: "VALIDATOR".into(),
            msg_chat_id: env::info_chat_id(),
            alert_chat_id: env::alert_chat_id(),
        };

        client_tg::init_with_client(
            settings,
            HttpProviderFactory::http_client()
                .expect("Failed to create http client"),
        );
    }

    // Wallet Data
    let pk = arc!(ValidatorSigner::new(env::validator_pk()).expect("PrivateKey hex is not valid!"));

    let url = env::eth_node_url();
    let version = env::validator_version();
    let validator = pk.address();

    // Low level providers
    let client = HttpProviderFactory::http_client()
        .expect("Failed to create http client");
    let rpc_url = url.parse()
        .expect("Failed to parse rpc_node_url");
    let web3 = arc!(Web3ProviderFactory::provider(rpc_url, env::chain_id(), &client, pk.wallet()));

    let db = arc!(
        SqliteClient::create(env::sqlite_path())
            .await
            .expect("Failed to create SQLite DB")
    );

    // High level providers
    let greenfield = arc!(GreenfieldClient::new(client.clone(), env::gf_node_url(), Some(pk.clone())));

    let store_service = arc!(ScStoreService::new(env::openstore_address(), version, &web3));
    let obj_service = arc!(ScObjService::new(web3.clone()));
    let validation_repo = arc!(ValidationRepo::new(&db));

    let verifier = arc!(ApkVerifierV2::new(ApkParser::default(), ApkChunker::default()));
    let build_verifier = arc!(AndroidBuildVerifier::new(&verifier));
    let file_storage = arc!(FileStorage::new(env::file_storage_path()));
    let android_validator = arc!(
        AndroidValidator::new(
            &greenfield,
            &build_verifier,
            &file_storage,
            obj_service.clone(),
            validation_repo.clone(),
        )
    );
    let block_repo = arc!(
        BlockRepo::new(
            version,
            validator.clone(),
        )
    );

    if env::is_dev() {
        validation_repo.clear_all()
            .await
            .expect("Failed to clear validation repo");
    }
    
    // Handlers
    let validation_case = arc!(
        ValidationCase::new(
            validation_repo.clone(),
            store_service.clone(),
            block_repo.clone(),
            android_validator.clone(),
        )
    );
    
    let top_up_case = arc!(
        TopUpCase::new(
            validator,
            store_service.clone(),
        )
    );

    let observe_voting = arc!(
        ObserveVotingHandler::new(
            store_service.clone(),
        )
    );

    let poll_config = ValidatorEventPoolConfig {
        filter_block_threshold: env::historical_sync_threshold(),
        timeout: Duration::from_millis(env::sync_retry_ms()),
        dry_timeout: Duration::from_millis(env::sync_timeout_ms()),
        topics: ScStoreService::validation_topics(),
        address: env::openstore_address(),
    };

    let poll = arc!(
        PollHandler::new(
            poll_config,
            web3.clone(),
            android_validator.clone(),
            validation_repo.clone(),
        )
    );

    let register = arc!(
        RegisterHandler::new(
            validator,
            version,
            store_service.clone(),
            top_up_case.clone(),
        )
    );

    let sync = arc!(
        SyncHandler::new(
            validator,
            store_service.clone(),
        )
    );

    let enqueue = arc!(
        TryAssignHandler::new(
            validator,
            store_service.clone(),
            top_up_case.clone(),
        )
    );

    let check_proposal = arc!(
        CheckProposalHandler::new(
            validator,
            store_service.clone(),
        )
    );

    let create_proposal_case = arc!(
        CreateProposalCase::new(
            validation_repo.clone(),
            store_service.clone(),
            block_repo.clone(),
        )
    );

    let validate = arc!(
        ValidateSyncHandler::new(
            validation_repo.clone(),
            store_service.clone(),
            android_validator.clone(),
            validation_case.clone(),
        )
    );

    let vote = arc!(
        VoteHandler::new(
            validator,
            validation_repo.clone(),
            store_service.clone(),
            create_proposal_case.clone(),
            validation_case.clone(),
        )
    );

    let propose = arc!(
        ProposeHandler::new(
            validator,
            validation_repo.clone(),
            store_service.clone(),
            create_proposal_case.clone(),
            validation_case.clone(),
        )
    );

    let observe_overdue = arc!(
        ObserveOverdueHandler {  }
    );

    let finalize = arc!(
        FinalizeHandler::new(
            validator,
            validation_repo.clone(),
            store_service.clone(),
        )
    );

    let unregister = arc!(
        UnregisterHandler::new(
            validator,
            store_service.clone(),
        )
    );

    let validator = arc!(
        ValidatorQueue::new(
            register.clone(),
            sync.clone(),
            poll.clone(),

            enqueue.clone(),
            validate.clone(),
            check_proposal.clone(),
            vote.clone(),
            propose.clone(),
            observe_voting.clone(),
            observe_overdue.clone(),
            finalize.clone(),

            unregister.clone(),
        )
    );

    // Launch queue
    let queue = arc!(ValidationQueue::new(10_000));
    let e_queue = queue.clone();
    tokio::spawn(async move {
        tokio::select! {
            _ = signal::ctrl_c() => {
                warn!("Shutdown validator queue");
                e_queue.push_sequential(ValidatorEvent::Unregister)
                    .await;
            },
        }
    });

    let queue_c = queue.clone();
    let validator = validator.clone();
    let validation_task = tokio::spawn(async move {
        queue_c.push(ValidatorEvent::Launch)
            .await;

        let _ = queue_c.run(validator)
            .await;
    });

    let _ = tokio::join!(validation_task);

    while !queue.is_shutdown_finished() {
        warn!("Waiting for parallel tasks shutdown signal");
        sleep(Duration::from_secs(5)).await;
    }
}

mod result;
mod data;
mod launcher;
pub mod env;
mod handler;
mod utils;

use crate::data::stat_buffer::StatBuffer;
use crate::data::stat_repo::StatRepo;
use crate::handler::event;
use crate::result::StatError;
use axum::extract::DefaultBodyLimit;
use axum::routing::post;
use axum::Router;
use core_std::arc;
use db_ch::client::ChClient;
use db_kf::client::{KfConsumer, KfProducer};
use dotenvy::dotenv;
use env::stat_max_body_size;
use rdkafka::Message;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing::info;

#[derive(Clone)]
struct AppState {
    state_repo: Arc<StatBuffer>,
}

#[tokio::main]
async fn main() {
    info!("Starting application");
    dotenv().ok();
    let _guard = core_log::init_tracer();

    let producer = arc!(
        KfProducer::new_client(
            env::kf_broker(),
            Some(env::kf_client()),
        ).expect("Failed to create kafka producer")
    );

    let consumer = arc!(
        KfConsumer::new_client(
            env::kf_broker(),
            env::kf_group(),
        ).expect("Failed to create kafka consumer")   
    );

    let state_buffer = arc!(
        StatBuffer::new(
            env::kf_topic(),
            env::kf_key(),
            producer,
            consumer
        )
    );

    let ch_client = arc!(
        ChClient::new_client(
            env::ch_url(),
            Some(env::ch_db()),
            Some(env::ch_user()),
            Some(env::ch_pass()),
        )
    );

    let stat_repo = arc!(
        StatRepo::new(ch_client.clone())
    );

    launcher::launch_consumer(
        state_buffer.clone(),
        stat_repo.clone()
    );

    let app_state = arc!(
        AppState { state_repo: state_buffer.clone() }
    );

    let app = Router::new()
        .route("/v1/event/create", post(event::create_event))
        .layer(DefaultBodyLimit::max(stat_max_body_size()))
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    let addr_str = env::stat_host_url();
    let addr: SocketAddr = addr_str.parse()
        .expect("Failed to parse socket address");

    info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind socket");

    axum::serve(listener, app.into_make_service())
        .await
        .expect("Failed to start server");
}


use axum::{routing::{get, post}, Router};
use client::data::models::Artifact;
use client::data::repo::artifact_repo::ArtifactRepo;
use client::data::repo::cache_repo::CacheRepo;
use client::data::repo::category_repo::CategoryRepo;
use client::data::repo::object_repo::ObjectRepo;
use client::data::repo::publishing_repo::PublishingRepo;
use client::data::repo::report_repo::ReportRepo;
use client::data::repo::review_repo::ReviewRepo;
use client::data::repo::search_repo::SearchRepo;
use client::data::repo::validation_repo::ValidationRepo;
use client::env::{psql_url, redis_url};
use client::net::etag_handler::EtagHandler;
use client::state::ClientState;
use client::{env, handler};
use cloud_gf::client::GreenfieldClient;
use cloud_gf::data::ObjectInfo;
use cloud_gf::proto::{QueryHeadObjectRequest, QueryHeadObjectResponse};
use core_std::arc;
use db_psql::client::PgClient;
use db_redis::cache::RedisCache;
use db_redis::client::RedisClient;
use dotenvy::dotenv;
use net_client::http::HttpProviderFactory;
use prost::Message;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info;
use core_log::init_tracer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting application");
    dotenv().ok();
    let _guard = init_tracer();

    // Optional: Run Migrations (ensure migrations folder exists)
    // db_client.migrate().await.map_err(ClientError::Migration)?;
    // info!("Database migrations checked/run.");

    info!("Connecting to databases...");
    let pg_client = PgClient::connect(psql_url().as_ref())
        .await
        .expect("Failed to connect to database");

    let client = RedisClient::new(redis_url())
        .expect("Failed to connect to redis");
    let redis_client = arc!(RedisCache::new(client));
    info!("Database connected successfully.");

    // --- Application State ---
    let cache = arc!(CacheRepo::new(redis_client));
    let publishing_repo =  arc!(PublishingRepo::new(pg_client.clone()));
    let artifact_repo = arc!(ArtifactRepo::new(pg_client.clone()));
    let object_repo = arc!(ObjectRepo::new(pg_client.clone()));
    let validation_repo = arc!(ValidationRepo::new(pg_client.clone()));
    
    let state = ClientState {
        object_repo: object_repo.clone(),
        category_repo: arc!(CategoryRepo::new(pg_client.clone())),
        search_repo: arc!(SearchRepo::new(pg_client.clone())),
        publishing_repo: publishing_repo.clone(),
        review_repo: arc!(ReviewRepo::new(pg_client.clone())),
        artifact_repo: artifact_repo.clone(),
        validation_repo: validation_repo.clone(),
        report_repo: arc!(ReportRepo::new(pg_client.clone())),
        etag_handler: arc!(EtagHandler::new(cache)),
    };

    info!("Application state created.");

    // --- CORS Configuration ---
    let cors = CorsLayer::new()
        // .allow_origin(Any) // Or specify allowed origins
        .allow_origin(tower_http::cors::Any) // Example: Allow any origin
        .allow_methods(tower_http::cors::Any) // Allow all methods or specify
        .allow_headers(tower_http::cors::Any); // Allow all headers or specify

    // --- API Routes ---
    let api_router = Router::new()
        .nest("/v1", v1_routes()); // Group all v1 routes

    // --- Main Router ---
    let app = Router::new()
        .merge(api_router)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state);

    // --- Server ---
    let addr: SocketAddr = env::client_host_url()
        .parse()?;

    info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await?;

    axum::serve(listener, app.into_make_service())
        .await?;

    return Ok(())
}

// Define v1 routes
fn v1_routes() -> Router<ClientState> {
    Router::new()
        // Admin Routes (Consider adding auth middleware layer)
        .route("/admin/set_categories", post(handler::admin::set_categories)) // reset etag - no-ttl
        // Store Routes
        .route("/feed", get(handler::store::get_feed)) // get-set etag - no-ttl
        .route("/store/categories", get(handler::store::get_categories)) // get-set etag - no-ttl
        // Object Route
        .route("/asset/chart", get(handler::store::get_chart)) // get-set etag - 1 hour
        .route("/asset/id/{asset_id}", get(handler::object::get_object_by_id)) // get-set etag
        .route("/asset/address/{address}", get(handler::object::get_object_by_address)) // get-set etag
        .route("/asset/search", get(handler::search::search_objects))
        .route("/asset/status/{address}", get(handler::object::get_object_status_by_address))
        // Artifact
        .route("/asset/{asset_id}/{track_id}/artifact", get(handler::artifact::get_artifact))
        // Review Routes
        .route("/review/{asset_id}", get(handler::review::get_reviews_for_object))
        .route("/review/create", post(handler::review::create_review))
        // Report Route
        .route("/report/create", post(handler::report::create_report))
        // Utils
        .route("/health", get(handler::util::handle_health))
}

use crate::env;
use crate::result::{AssetlinkError, AssetlinkResult};
use crate::verifier::android::app_verifier::AndroidAppVerifier;
use crate::verifier::app_verifier::AppVerifier;
use crate::verifier::asset_provider::AssetProvider;
use alloy::primitives::Address;
use alloy::providers::Provider;
use alloy::rpc::json_rpc::SerializedRequest;
use alloy::rpc::types::{Filter, Log};
use alloy::sol_types::sol_data::{Int, Uint};
use alloy::sol_types::{sol_data, SolType};
use async_trait::async_trait;
use client_tg::{tg_alert, tg_msg};
use core_actor::{ActionQueue, ActionQueueError, Context, EventHandler, UniqueEvent};
use core_std::arc;
use core_std::trier::SyncTrier;
use derive_more::Display;
use hex::ToHex;
use net_client::node::provider::Web3Provider;
use net_client::node::result::EthResult;
use service_sc::assetlinks::{AssetlinkStatusCode, ScAssetLinkService, VerificationState};
use service_sc::obj::{ObjOwnerDataV1, ScObjService};
use std::cmp::max;
use std::future::Future;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tracing::{error, info, warn};

#[derive(Debug, Display, Clone, Hash, Eq, PartialEq)]
pub enum OracleEvent {
    #[display("DownloadsRecount")]
    Poll,
}

#[derive(Display)]
enum OracleBuildStage {
    #[display("AppInfo")]
    AppInfo,
    #[display("OwnerData")]
    OwnerData(AppInfo),
    #[display("Verify")]
    Verify(AppInfo, ObjOwnerDataV1),
    #[display("Finish")]
    Finish(VerificationState)
}

#[derive(Debug, Clone)]
struct AppInfo {
    id: String,
    address: Address,
    owner_version: u64,
}

impl UniqueEvent<u64> for OracleEvent {

    fn event_id(&self) -> u8 {
        match self {
            OracleEvent::Poll => 1,
        }
    }

    fn unique_key(&self) -> Option<(u64)> {
        match self {
            OracleEvent::Poll => Some(0),
        }
    }
}

pub type OracleQueue = ActionQueue<u64, OracleEvent>;
pub type OracleCtx = Context<u64, OracleEvent>;

pub struct OracleHandler {
    protocol_version: u64,
    timeout: u64,
    empty_timeout: u64,
    asset_provider: Arc<ScAssetLinkService>,
    app_provider: Arc<ScObjService>,
    app_verifier: Arc<AndroidAppVerifier>,
}

#[async_trait]
impl EventHandler<u64, OracleEvent> for OracleHandler {
    async fn handle(&self, event: OracleEvent, ctx: Arc<OracleCtx>) -> Result<(), ActionQueueError> {
        match event {
            OracleEvent::Poll => {
                self.handle_poll(ctx.clone())
                    .await;
            }
        }

        return Ok(())
    }
}

impl OracleHandler {

    pub fn new(
        oracle_version: u64,
        timeout: u64,
        empty_timeout: u64,
        asset_provider: &Arc<ScAssetLinkService>,
        app_provider: &Arc<ScObjService>,
        app_verifier: &Arc<AndroidAppVerifier>
    ) -> Self {
        return Self {
            protocol_version: oracle_version, timeout, empty_timeout,
            asset_provider: asset_provider.clone(),
            app_provider: app_provider.clone(),
            app_verifier: app_verifier.clone(),
        };
    }

    async fn handle_poll(&self, ctx: Arc<OracleCtx>) {
        info!("[ORACLE_POOL] Oracle START polling...");
        tg_msg!("[ORACLE_POOL] Oracle START polling...");

        loop {
            if ctx.queue.is_shutdown() {
                warn!("[ORACLE_POOL] Shutdown oracle!");
                break;
            }

            let result = self.asset_provider.get_state().await;

            let (last_req, next_queue_req, _) = match result {
                Ok(state) => state,
                Err(e) => {
                    error!("[ORACLE_POOL] Oracle can't get state: {}", e);
                    tg_msg!(format!("Can't get state from Node {}", e));

                    sleep(Duration::from_secs(self.timeout))
                        .await;
                    
                    continue;
                }
            };

            let next_val_req = last_req + 1;
            if next_queue_req > next_val_req { // 1 is starting value(empty queue)
                for req in next_val_req..next_queue_req {
                    if let Err(e) = self.handle_request(req).await {
                        error!("[ORACLE_POOL] Oracle can't finalize request: {}", e);
                        tg_alert!(format!("Oracle can't handle request, shutting down! {}", e));

                        ctx.queue.async_shutdown().await;
                        break;
                    };
                }
                info!("[ORACLE_POOL] Oracle sleeping...");
                sleep(Duration::from_secs(self.timeout))
                    .await;
            } else {
                info!("[ORACLE_POOL] Oracle is empty, sleeping...");
                sleep(Duration::from_secs(self.empty_timeout))
                    .await;
            }
        }
    }

    async fn handle_request(&self, request_id: i64) -> AssetlinkResult<()> {
        let mut trier = SyncTrier::new(10, 1.0, 6);
        let mut stage = OracleBuildStage::AppInfo;

        'trier: while trier.iterate().await {
            loop {
                stage = match stage {
                    OracleBuildStage::AppInfo => {
                        let result = self.asset_provider.get_app_from_queue(request_id)
                            .await;

                        let (app, owner_version) = match result {
                            Ok(state) => state,
                            Err(e) => {
                                if trier.is_last() {
                                    error!("[ORACLE_POOL] Oracle owner request [{}] error, last try: {}", stage, e);
                                    trier.reset();
                                    stage = OracleBuildStage::Finish(VerificationState { status: AssetlinkStatusCode::ExceedRpcAttemptsErrors });
                                    continue;
                                } else {
                                    error!("[ORACLE_POOL] Oracle request [{}] error: {}", stage, e);
                                    continue 'trier
                                }
                            }
                        };

                        let app_package = self.app_provider.obj_package(app)
                            .await;

                        match app_package {
                            Ok(app_package) => OracleBuildStage::OwnerData(AppInfo { address: app, owner_version, id: app_package }),
                            Err(e) => {
                                if trier.is_last() {
                                    error!("[ORACLE_POOL] Oracle package request [{}] error, last try: {}", stage, e);
                                    trier.reset();
                                    OracleBuildStage::Finish(VerificationState { status: AssetlinkStatusCode::ExceedRpcAttemptsErrors })
                                } else {
                                    error!("[ORACLE_POOL] Oracle request [{}] error: {}", stage, e);
                                    continue 'trier
                                }
                            }
                        }
                    }
                    OracleBuildStage::OwnerData(ref app_info) => {
                        info!("[ORACLE_POOL] Oracle request stage: {}, {}", stage, request_id);

                        let data = match self.protocol_version {
                            0 => {
                                self.app_provider.get_owner_data_v0(
                                    app_info.address, app_info.owner_version.clone()
                                ).await
                            },
                            _ => {
                                self.app_provider.get_owner_data_v1(
                                    app_info.address, app_info.owner_version.clone()
                                ).await
                            }
                        };

                        match data {
                            Ok(data) => OracleBuildStage::Verify(app_info.clone(), data),
                            Err(e) => {
                                if trier.is_last() {
                                    error!("[ORACLE_POOL] Oracle owner request [{}] error, last try: {}", stage, e);
                                    trier.reset();
                                    OracleBuildStage::Finish(VerificationState { status: AssetlinkStatusCode::ExceedRpcAttemptsErrors })
                                } else {
                                    error!("[ORACLE_POOL] Oracle request [{}] error: {}", stage, e);
                                    continue 'trier
                                }
                            }
                        }
                    }
                    OracleBuildStage::Verify(ref app_info, ref data) => {
                        info!("[ORACLE_POOL] Oracle request stage: {}, {}", stage, request_id);
                        let fingerprints: Vec<String> = data.fingerprints.iter()
                            .map(|finger| finger.encode_hex_upper())
                            .collect();

                        let app_result = self.app_verifier.verify(
                            app_info.id.clone(),
                            app_info.owner_version.clone(),
                            data.website.clone(),
                            fingerprints
                        ).await;

                        trier.reset();

                        let new_state = VerificationState { status: app_result.new_status };
                        OracleBuildStage::Finish(new_state)
                    }
                    OracleBuildStage::Finish(ref state) => {
                        info!("[ORACLE_POOL] Oracle request stage: {}, {}, {}", stage, request_id, state.status);
                        let result = self.asset_provider.finish(request_id, state)
                            .await;
                        
                        if let Err(e) = result {
                            if trier.is_last() {
                                error!("[ORACLE_POOL] Oracle request [{}] error, last try: {}", stage, e);
                                return Err(AssetlinkError::CantFinalize)
                            } else {
                                error!("[ORACLE_POOL] Oracle request [{}] error: {}", stage, e);
                                continue 'trier
                            }
                        }

                        info!("[ORACLE_POOL] Oracle request [{}] success!", stage);
                        break 'trier;
                    }
                }
            }
        }

        return Ok(())
    }
}

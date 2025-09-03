use std::sync::Arc;
use alloy::primitives::Address;
use tracing::{error, info};
use client_tg::tg_alert;
use service_sc::store::ScStoreService;
use crate::launcher::ValidationContext;
use core_std::trier::SyncTrier;

pub struct UnregisterHandler {
    validator_address: Address,
    service: Arc<ScStoreService>,
}

enum UnregisterStages {
    Unassign,
    Unregister,
}

impl UnregisterHandler {

    pub fn new(
        validator_address: Address,
        service: Arc<ScStoreService>
    ) -> Self {
        Self {
            validator_address,
            service
        }
    }

    pub async fn handle(&self, ctx: Arc<ValidationContext>) {
        info!("[UNREGISTER] Unregister validator...");
        tg_alert!("[UNREGISTER] Unregister validator...");

        // TODO test no more events, check race conditions etc...
        ctx.queue.async_shutdown()
            .await;
        
        let mut trier = SyncTrier::new(30, 1.0, 10_000);
        let mut stage = UnregisterStages::Unassign;

        'retrier: while trier.iterate().await {
            stage = match stage {
                UnregisterStages::Unassign => {
                    let result = self.service.get_state(self.validator_address).await;
                    let state = match result {
                        Ok(state) => state,
                        Err(err) => {
                            error!("[UNREGISTER] Can't get store state: {}", err);
                            tg_alert!(format!("[UNREGISTER] Can't get store state: {}", err));
                            continue 'retrier;
                        }
                    };

                    if state.can_unassign() {
                        let result = self.service.unassign_validator(state.my_block).await;
                        if let Err(e) = result {
                            error!("[UNREGISTER] Can't unassign validator: {}", e);
                            tg_alert!(format!("[UNREGISTER] Can't unassign validator: {}", e));
                            continue 'retrier;
                        };
                        info!("[UNREGISTER] Unassigned validator!");
                        tg_alert!("[UNREGISTER] Unassigned validator!");
                    } else {
                        info!("[UNREGISTER] Unassign is not needed!");
                        tg_alert!("[UNREGISTER] Unassign is not needed!");
                    }

                    UnregisterStages::Unregister
                }
                UnregisterStages::Unregister => {
                    let result = self.service.is_registered(self.validator_address)
                        .await;

                    match result {
                        Ok(is_registered) => {
                            if !is_registered {
                                info!("[UNREGISTER] Validator is not registered! Finalizing validator...");
                                tg_alert!("[UNREGISTER] Validator is not registered! Finalizing validator...");
                                break 'retrier;
                            }
                        },
                        Err(err) => {
                            error!("[UNREGISTER] Error during getting register status: {}", err);
                            tg_alert!(format!("[UNREGISTER] Error during getting register status: {}", err));
                            continue 'retrier;
                        }
                    };

                    let result = self.service.unregister_validator()
                        .await;

                    if let Err(e) = result {
                        error!("[UNREGISTER] Unregister validator error: {}", e);
                        tg_alert!(format!("[UNREGISTER] Unregister validator error: {}", e));
                        continue 'retrier;
                    }

                    info!("[UNREGISTER] Validator is unregistered! Finalizing validator...");
                    tg_alert!("[UNREGISTER] Validator is unregistered! Finalizing validator...");
                    break 'retrier;
                }
            }
        }

        if trier.is_exceeded() {
            error!("[UNREGISTER] Validator exceeded error!");
            tg_alert!("[UNREGISTER] Validator exceeded error!");
        }
    }
}

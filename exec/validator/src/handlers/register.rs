use std::sync::Arc;
use alloy::primitives::Address;
use tracing::{error, info};
use client_tg::{tg_alert, tg_msg};
use service_sc::obj::ScObjService;
use service_sc::store::ScStoreService;
use crate::handlers::common::top_up::TopUpCase;
use crate::launcher::{ValidationContext, ValidatorEvent};
use core_std::trier::SyncTrier;
use net_client::node::result::EthResult;

enum RegisterStages {
    CheckingVersion,
    CheckingRegistration,
    TopUp,
    Register,
}

pub struct RegisterHandler {
    validator_address: Address,
    validator_version: u64,
    service: Arc<ScStoreService>,
    handler: Arc<TopUpCase>,
}

impl RegisterHandler {

    pub fn new(
        validator_address: Address,
        validator_version: u64,
        service: Arc<ScStoreService>,
        handler: Arc<TopUpCase>,
    ) -> Self {
        Self {
            validator_address,
            validator_version,
            service,
            handler
        }
    }

    pub async fn handle(&self, ctx: Arc<ValidationContext>) {
        info!("[REGISTER] Register validator...");

        let mut trier = SyncTrier::new(30, 1.0, 4);
        let mut stage = RegisterStages::CheckingVersion;

        'trier: while trier.iterate().await {
            loop {
                stage = match stage {
                    RegisterStages::CheckingVersion => {
                        let min_version = self.service.min_available_version()
                            .await;

                        match min_version {
                            Ok(min_version) => {
                                if min_version > self.validator_version {
                                    error!("[REGISTER] Validator version is lower than required!");
                                    tg_alert!("[REGISTER] Validator version is lower than required!");

                                    ctx.queue.push_sequential(ValidatorEvent::Unregister)
                                        .await;

                                    return;
                                }
                            }
                            Err(e) => {
                                error!("[REGISTER] Checking validator version error: {}", e);
                                tg_alert!(format!("[REGISTER] Checking validator version error: {}", e));
                                continue 'trier
                            }
                        }

                        RegisterStages::CheckingRegistration
                    }
                    RegisterStages::CheckingRegistration => {
                        let result = self.service.is_registered(self.validator_address)
                            .await;

                        let Ok(is_registered) = result else {
                            let e = result.unwrap_err();
                            error!("[REGISTER] Checking register validator error: {}", e);
                            tg_alert!(format!("[REGISTER] Checking register validator error: {}", e));
                            continue 'trier
                        };

                        if is_registered {
                            info!("[REGISTER] Validator is already registered! Starting validator...");
                            tg_msg!("[REGISTER] Validator is already registered! Starting validator...");
                            break 'trier;
                        }

                        RegisterStages::TopUp
                    }
                    RegisterStages::TopUp => {
                        if !self.handler.handle().await {
                            continue 'trier
                        }

                        RegisterStages::Register
                    }
                    RegisterStages::Register => {
                        let result = self.service.register_validator()
                            .await;

                        if let Err(e) = result {
                            error!("[REGISTER] Register validator error: {}", e);
                            tg_alert!(format!("[REGISTER] Register validator error: {}", e));

                            let result = self.service.is_registered(self.validator_address)
                                .await;

                            let Ok(is_registered) = result else {
                                let e = result.unwrap_err();
                                error!("[REGISTER] Checking register validator error: {}", e);
                                tg_alert!(format!("[REGISTER] Checking register validator error: {}", e));
                                continue 'trier
                            };

                            if !is_registered {
                                error!("[REGISTER] Validator error and it's not registered yet!");
                                tg_msg!("[REGISTER] Validator error and it's not registered yet!");
                                continue 'trier
                            }
                        }

                        info!("[REGISTER] Validator is registered! Starting validator...");
                        tg_msg!("[REGISTER] Validator is registered! Starting validator...");
                        break 'trier;
                    }
                }
            }
        }

        if trier.is_failed() {
            error!("[REGISTER] Validator exceeded error");
            ctx.queue.push_sequential(ValidatorEvent::Unregister)
                .await;
            tg_alert!("[REGISTER] Validator exceeded error!");
        } else {
            ctx.queue.push_sequential(ValidatorEvent::Sync)
                .await;
        }
    }
}

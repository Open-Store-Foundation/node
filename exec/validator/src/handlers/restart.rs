use std::sync::Arc;
use alloy::primitives::Address;
use tracing::{error, info};
use client_tg::tg_alert;
use service_sc::store::ScStoreService;
use crate::launcher::ValidationContext;
use core_std::trier::SyncTrier;

#[derive(Default)]
pub struct RestartHandler;

impl RestartHandler {

    pub async fn handle(&self, ctx: Arc<ValidationContext>) {
        ctx.queue.async_shutdown()
            .await;

        info!("[RESTART] Validator is restarting! Finalizing validator...");
        tg_alert!("[RESTART] Validator is restarting! Finalizing validator...");
    }
}

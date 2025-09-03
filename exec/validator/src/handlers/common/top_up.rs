use alloy::primitives::{Address};
use service_sc::store::{ScStoreService, StoreBlockRef};
use std::sync::Arc;
use tracing::{error};
use client_tg::tg_alert;
use codegen_block::block::{ValidationResult};

pub struct TopUpCase {
    validator_address: Address,
    service: Arc<ScStoreService>,
}

impl TopUpCase {

    pub fn new(
        validator_address: Address,
        service: Arc<ScStoreService>,
    ) -> Self {
        Self {
            validator_address,
            service,
        }
    }

    pub async fn handle(&self) -> bool {
        let result = self.service.recommended_stake_amount()
            .await;

        let recommended_stake = match result {
            Ok(val) => val,
            Err(err) => {
                error!("[REGISTER] Can't get recommended stake amount. Error: {}", err);
                tg_alert!(format!("[REGISTER] Can't get recommended stake amount. Error: {}", err));
                return false;
            }
        };

        let result = self.service.total_balance(self.validator_address)
            .await;

        let balance = match result {
            Ok(val) => val,
            Err(err) => {
                error!("[REGISTER] Can't get total balance. Error: {}", err);
                tg_alert!(format!("[REGISTER] Can't get total balance. Error: {}", err));
                return false;
            }
        };
        // TODO check wallet balance
        if balance < recommended_stake {
            let result = self.service.top_up(recommended_stake - balance)
                .await;

            if let Err(err) = result {
                error!("[REGISTER] Can't top up. Error: {}", err);
                tg_alert!(format!("[REGISTER] Can't top up. Error: {}", err));
                return false;
            }
        }

        return true;
    }
}

use once_cell::sync::OnceCell;
use reqwest::Client;
use crate::client::{TgClient, TgClientSettings};

pub mod client;

pub static GLOBAL_TG_CLIENT: OnceCell<TgClient> = OnceCell::new();

// TODO add logs
pub fn init_with_client(settings: TgClientSettings, client: Client) {
    if GLOBAL_TG_CLIENT.get().is_none() {
        let tg_bot = TgClient::with_client(settings, client);
        if GLOBAL_TG_CLIENT.set(tg_bot).is_err() {
            eprintln!("WARNING: Attempted to initialize Telegram client more than once.");
        }
    }
}

#[macro_export]
macro_rules! tg_msg {
    ($value:expr) => {
        if let Some(client) = $crate::GLOBAL_TG_CLIENT.get() {
            client.send_error($value).await;
        }
    };
}

#[macro_export]
macro_rules! tg_alert {
    ($value:expr) => {
        if let Some(client) = $crate::GLOBAL_TG_CLIENT.get() {
            client.send_alert($value).await;
        }
    };
}
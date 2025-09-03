use reqwest::Client;
use serde_json::json;

pub struct TgClientSettings {
    pub token: String,
    pub client_name: String,
    pub msg_chat_id: i64,
    pub alert_chat_id: i64,
}

pub struct TgClient {
    client_name: String,
    msg_chat_id: i64,
    alert_chat_id: i64,
    client: Client,
    bot_token: String,
}

impl TgClient {
    
    pub fn with_client(settings: TgClientSettings, client: Client) -> Self {
        Self {
            client_name: settings.client_name,
            msg_chat_id: settings.msg_chat_id,
            alert_chat_id: settings.alert_chat_id,
            client,
            bot_token: settings.token,
        }
    }

    async fn send_message(&self, chat_id: i64, message: String) -> Result<(), reqwest::Error> {
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.bot_token);
        
        let payload = json!({
            "chat_id": chat_id,
            "text": message
        });

        self.client
            .post(&url)
            .json(&payload)
            .send()
            .await?;

        Ok(())
    }

    pub async fn send_error<T: Into<String>>(&self, message: T) {
        let formatted_message = format!("[{}]: {}", self.client_name, message.into());
        let _ = self.send_message(self.msg_chat_id, formatted_message).await;
    }

    pub async fn send_alert<T: Into<String>>(&self, message: T) {
        let formatted_message = format!("[{}]: {}", self.client_name, message.into());
        let _ = self.send_message(self.alert_chat_id, formatted_message).await;
    }
}

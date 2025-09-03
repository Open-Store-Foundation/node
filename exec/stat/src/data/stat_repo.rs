use crate::data::models::ObjectEvent;
use clickhouse::insert::Insert;
use db_ch::client::ChClient;
use std::sync::Arc;
use tracing::instrument;


#[derive(Clone)]
pub struct StatRepo {
    client: Arc<ChClient>,
}

impl StatRepo {

    pub fn new(client: Arc<ChClient>) -> Self {
        Self { client }
    }

    #[instrument(skip(self, events), fields(count = events.len()))]
    pub async fn insert_events(&self, events: Vec<ObjectEvent>) -> clickhouse::error::Result<()> {

        let mut insert: Insert<ObjectEvent> = self.client.client
            .insert("default.stat_events")?;

        for event in events {
            insert.write(&event)
                .await?;
        }

        insert.end()
            .await?;

        Ok(())
    }
}
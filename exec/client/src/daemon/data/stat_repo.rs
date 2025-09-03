use chrono::{DateTime, Utc};
use clickhouse::Row;
use db_ch::client::ChClient;
use db_psql::client::{PgClient, SqlxResult};
use serde::Deserialize;
use tracing::instrument;

#[derive(Row, Deserialize, Debug, Clone)]
pub struct ObjectSummary {
    object_id: i64,
    install_count: u64,
}

pub struct StatSyncHandler {
    ch_client: ChClient,
    pg_client: PgClient,
}

impl StatSyncHandler {
    
    pub async fn get_summary_since(
        &self,
        last_event_time: DateTime<Utc>,
    ) -> clickhouse::error::Result<Vec<ObjectSummary>> {
        let query = "
            SELECT
                object_id,
                count(*) AS install_count
            FROM default.stat_events
            WHERE
                event_name = 'ObjectInstalled' AND event_time > ?
            GROUP BY object_id
            ORDER BY object_id
        ";

        let mut cursor = self.ch_client
            .client
            .query(query)
            .bind(last_event_time)
            .fetch::<ObjectSummary>()?;

        let mut results = Vec::new();
        
        while let Some(summary) = cursor.next().await? {
            results.push(summary);
        }

        Ok(results)
    }

    #[instrument(skip(self, updates), fields(count = updates.len()))]
    pub async fn increment_downloads(
        &self,
        updates: Vec<ObjectSummary>,
    ) -> SqlxResult<()> {
        // let mut tx = self.pg_client.start()
        //     .await?;
        // 
        // for update in updates {
        //     sqlx::query!(
        //         "UPDATE obj SET downloads = downloads + $1 WHERE id = $2",
        //         update.install_count as i64, // Cast u64 to i64 for SQL query parameter
        //         update.object_id
        //     )
        //         .execute(&mut *tx)
        //         .await?;
        // }
        // 
        // tx.commit()
        //     .await?;

        Ok(())
    }
}

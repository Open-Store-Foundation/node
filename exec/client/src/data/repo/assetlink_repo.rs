use crate::data::models::AssetlinkSync;
use crate::result::ClientResult;
use db_psql::client::PgClient;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AssetlinkRepo {
    client: PgClient,
}

impl AssetlinkRepo {

    pub fn new(client: PgClient) -> Self {
        Self { client }
    }

    pub fn pool(&self) -> &PgPool {
        self.client.pool()
    }
    
    pub async fn insert_assetlink_status(&self, data: &AssetlinkSync) -> ClientResult<()> {
        let result = sqlx::query_as!(
            AssetlinkSync,
            r#"
            INSERT INTO assetlink_sync (
                object_address, domain, owner_version, status
            )
            VALUES ($1, $2, $3, $4)
            "#,
            &data.object_address,
            &data.domain,
            data.owner_version.clone() as i64,
            data.status.clone() as i32
        )
            .execute(self.pool())
            .await?;

        Ok(())
    }
}
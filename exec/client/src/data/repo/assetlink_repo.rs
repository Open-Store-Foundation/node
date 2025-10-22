use crate::data::models::{AssetlinkSync, ValidationProof};
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

    pub async fn insert_validation_proof(&self, data: &ValidationProof) -> ClientResult<()> {
        let result = sqlx::query_as!(
            ValidationProof,
            r#"
            INSERT INTO validation_proof (
                object_address, owner_version, status
            )

            VALUES ($1, $2, $3)
            ON CONFLICT DO NOTHING;
            "#,
            &data.object_address,
            data.owner_version.clone() as i64,
            data.status.clone() as i32
        )
            .execute(self.pool())
            .await?;

        Ok(())
    }
}
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

    pub async fn get_proof_status_by_address(&self, address: &String) -> ClientResult<Option<ValidationProof>> {
        let result = sqlx::query_as!(
            ValidationProof,
            r#"
            SELECT asset_address, owner_version, status FROM validation_proof
            WHERE asset_address = $1
            ORDER BY owner_version
            DESC
            LIMIT 1;
            "#,
            address
        )
            .fetch_optional(self.pool())
            .await?;

        return Ok(result);
    }

    pub async fn insert_assetlink_status(&self, data: &AssetlinkSync) -> ClientResult<()> {
        let result = sqlx::query_as!(
            AssetlinkSync,
            r#"
            INSERT INTO assetlink_sync (
                asset_address, domain, owner_version, status
            )
            VALUES ($1, $2, $3, $4)
            "#,
            &data.asset_address,
            &data.domain,
            data.owner_version,
            data.status
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
                asset_address, owner_version, status
            )

            VALUES ($1, $2, $3)
            ON CONFLICT DO NOTHING;
            "#,
            &data.asset_address,
            data.owner_version,
            data.status
        )
            .execute(self.pool())
            .await?;

        Ok(())
    }
}
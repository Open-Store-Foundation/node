use crate::data::dto::DtoPublishing;
use crate::data::models::{Artifact, Publishing};
use crate::result::ClientResult;
use alloy::hex::ToHexExt;
use alloy::primitives::Address;
use codegen_contracts::ext::ToChecksum;
use core_std::hexer;
use db_psql::client::PgClient;
use sqlx::PgPool;

#[derive(Clone)]
pub struct PublishingRepo {
    client: PgClient,
}

impl PublishingRepo {

    pub fn new(client: PgClient) -> Self {
        Self { client }
    }

    pub fn pool(&self) -> &PgPool {
        self.client.pool()
    }

    pub async fn insert_or_update(
        &self, 
        publishing: &Publishing
    ) -> ClientResult<()> {
        let track_id: i32 = publishing.track_id.clone().into();
        let row = sqlx::query_as!(
            Publishing,
            r#"
            INSERT INTO publishing (
                asset_address,
                track_id,
                version_code,
                is_active
            )
            
            VALUES ($1, $2, $3, $4)
            
            ON CONFLICT (asset_address, track_id) DO UPDATE SET
                version_code = EXCLUDED.version_code,
                is_active = EXCLUDED.is_active
            "#,
            publishing.asset_address.checksum(),
            track_id,
            publishing.version_code,
            publishing.is_active
        )
            .execute(self.pool())
            .await?;

        return Ok(())
    }
    
    pub async fn get_publishing_by_address(
        &self,
        address: &String,
    ) -> ClientResult<Vec<DtoPublishing>> {
        let rows = sqlx::query!(
            r#"
            SELECT DISTINCT ON (pub.track_id)
                pub.*,
                
                art.id AS artifact_id_,
                art.ref_id AS artifact_ref_id,
                art.asset_address AS artifact_asset_address,
                art.protocol_id AS artifact_protocol_id,
                art.size AS artifact_size,
                art.version_name AS artifact_version_name,
                art.version_code AS artifact_version_code,
                art.created_at AS artifact_created_at,
                art.checksum AS artifact_checksum,
                br.status AS "status!: i32"
            FROM publishing pub
                INNER JOIN artifact art ON art.asset_address = pub.asset_address
                INNER JOIN obj ON obj.address = pub.asset_address
                INNER JOIN build_request br ON br.asset_address = pub.asset_address
            WHERE pub.asset_address = $1 AND br.version_code = pub.version_code
            ORDER BY pub.track_id, br.created_at DESC;
            "#,
            address.checksum()
        )
            .fetch_all(self.pool())
            .await?;

        let mut publishings = Vec::new();
        for row in rows {
            let artifact = Artifact {
                id: row.artifact_id_,
                ref_id: row.artifact_ref_id,
                asset_address: row.artifact_asset_address,
                protocol_id: row.artifact_protocol_id,
                size: row.artifact_size,
                checksum: row.artifact_checksum,
                version_name: row.artifact_version_name,
                version_code: row.artifact_version_code,
            };

            let publishing_entry = DtoPublishing {
                id: Some(row.id),
                asset_address: row.asset_address,
                track_id: row.track_id,
                status: row.status,
                is_active: row.is_active,
                artifact, // The manually constructed artifact
            };

            publishings.push(publishing_entry);
        }

        Ok(publishings)
    }
}

use bytes::Bytes;
use crate::data::models::{Artifact, NewArtifact};
use crate::result::{ClientError, ClientResult};
use db_psql::client::PgClient;
use sqlx::{FromRow, PgPool};
use tracing::error;

#[derive(Clone)]
pub struct ArtifactRepo {
    client: PgClient,
}

impl ArtifactRepo {

    pub fn new(client: PgClient) -> Self {
        Self { client }
    }

    pub fn pool(&self) -> &PgPool {
        self.client.pool()
    }

    // TODO v2 optimize
    pub async fn find_artifact_missing_refs(
        &self,
        object_refs: Vec<(String, i64)>,
    ) -> Vec<(String, i64)> {
        if object_refs.is_empty() {
            return Vec::new();
        }
        
        let (addresses, versions): (Vec<String>, Vec<i64>) = object_refs
            .into_iter()
            .unzip();

        let rows = sqlx::query!(
            r#"
            SELECT
                input.set_address as "address!",
                input.set_version as "version!"
            
            FROM UNNEST($1::varchar(100)[], $2::bigint[]) AS input(set_address, set_version)
                
            LEFT JOIN artifact ON artifact.asset_address = input.set_address
                AND artifact.version_code = input.set_version
            
            WHERE artifact IS NULL
            "#,
            &addresses,
            &versions
        )
            .fetch_all(self.pool())
            .await;
        
        match rows {
            Ok(rows) => {
                rows.into_iter()
                    .map(|row| (row.address, row.version))
                    .collect()
            }
            Err(e) => {
                error!("Error fetching missing refs: {}", e);
                addresses.into_iter()
                    .zip(versions.into_iter())
                    .collect()
            }
        }
    }

    pub async fn insert_artifact(&self, data: &NewArtifact) -> ClientResult<()> {
        sqlx::query_as!(
            Artifact,
            r#"
            INSERT INTO artifact (
                ref_id, asset_address, protocol_id, size, version_name, version_code, checksum
            )
            
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            
            ON CONFLICT (asset_address, version_code) DO NOTHING
            "#,
            data.object_ref,
            data.asset_address,
            data.protocol_id,
            data.size as i64,
            data.version_name,
            data.version_code,
            data.checksum,
        )
            .execute(self.pool())
            .await?;

        Ok(())
    }

    pub async fn find_by_obj_track(&self, obj_id: i64, track_id: i32) -> ClientResult<Option<Artifact>> {
        let result = sqlx::query_as!(
            Artifact,
            r#"
            SELECT artifact.id, ref_id, artifact.asset_address, protocol_id, size, version_name, artifact.version_code, artifact.checksum
            
            FROM artifact
            INNER JOIN obj o ON o.id = $1
            INNER JOIN publishing p ON o.address = p.asset_address

            WHERE p.track_id = $2
            AND artifact.asset_address = o.address
            "#,
            obj_id,
            track_id
        )
            .fetch_optional(self.pool())
            .await?;

        return Ok(result)
    }

    /// Finds an artifact by its primary key (ID).
    pub async fn find_by_id(&self, arc_id: i64) -> ClientResult<Option<Artifact>> {
        let result = sqlx::query_as!(
            Artifact,
            r#"
            SELECT id, size, ref_id, asset_address, protocol_id, version_name, version_code, checksum
            FROM artifact
            
            WHERE id = $1
            "#,
            arc_id
        )
            .fetch_optional(self.pool())
            .await?;

        return Ok(result)
    }

    /// Deletes an artifact by its ID.
    pub async fn delete(&self, artifact_id: i64) -> ClientResult<u64> {
        let result = sqlx::query!(
            r#"
            DELETE FROM artifact 
            WHERE id = $1
            "#,
            artifact_id
        )
            .execute(self.pool())
            .await?;

        if result.rows_affected() == 0 {
            return Err(ClientError::NotFound)
        }

        Ok(result.rows_affected())
    }
}
use crate::data::id::{ReqTypeId, TrackId};
use crate::data::models::{BuildRequest, NewBuildRequest};
use crate::result::ClientResult;
use codegen_contracts::ext::ToChecksum;
use db_psql::client::PgClient;
use sqlx::PgPool;

#[derive(Clone)]
pub struct ValidationRepo {
    client: PgClient,
}

impl ValidationRepo {

    pub fn new(client: PgClient) -> Self {
        Self { client }
    }

    pub fn pool(&self) -> &PgPool {
        self.client.pool()
    }
    
    pub async fn get_requests_by_address(
        &self,
        address: String,
    ) -> ClientResult<Vec<BuildRequest>> {
        let rows = sqlx::query!(
        r#"
            SELECT
                id,
                request_type_id,
                asset_address,
                track_id,
                status,
                version_code,
                owner_version

            FROM build_request
                
            WHERE asset_address = $1 AND status = -1;
            "#,
            address.checksum()
        )
            .fetch_all(self.pool())
            .await?;

        let build_requests = rows
            .into_iter()
            .filter_map(|row| {
                Some(
                    BuildRequest {
                        id: row.id,
                        request_type_id: ReqTypeId::from(row.request_type_id),
                        track_id: TrackId::from(row.track_id),
                        owner_version: row.owner_version as u64,
                        asset_address: row.asset_address,
                        status: row.status,
                        version_code: row.version_code,
                    }
                )
            })
            .collect();

        Ok(build_requests)
    }

    pub async fn insert_or_update(&self, new_req: &NewBuildRequest) -> ClientResult<()> {
        let req_type_id: i32 = new_req.request_type_id.clone().into();
        let track_id: i32 = new_req.track_id.clone().into();
        sqlx::query_as!(
            BuildRequest,
            r#"
            INSERT INTO build_request (
                id,
                request_type_id,
                asset_address,
                track_id,
                status,
                version_code,
                owner_version,
                created_at
            )
            
            VALUES ($1, $2, $3, $4, $5, $6, $7, COALESCE($8, CURRENT_TIMESTAMP))
            
            ON CONFLICT (id) DO UPDATE SET
                status = EXCLUDED.status
            "#,
            new_req.id,
            req_type_id,
            new_req.asset_address.checksum(),
            track_id,
            new_req.status,
            new_req.version_code,
            new_req.owner_version as i64,
            new_req.created_at
        )
            .execute(self.pool())
            .await?;

        return Ok(())
    }
}
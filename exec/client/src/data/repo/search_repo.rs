use crate::data::models::Asset;
use crate::result::ClientResult;
use db_psql::client::PgClient;
use sqlx::PgPool;
use crate::data::id::ObjTypeId;

pub struct SearchRepo {
    client: PgClient,
}

impl SearchRepo {

    pub fn new(client: PgClient) -> Self {
        Self { client }
    }

    pub fn pool(&self) -> &PgPool {
        self.client.pool()
    }

    pub async fn search(
        &self,
        term: &str,
        platform_id: i32,
        type_id: Option<ObjTypeId>, // TODO split on app/game when it will be many apps
        limit: i64,
        offset: i64,
    ) -> ClientResult<Vec<Asset>> {
        let search_pattern = format!("%{}%", term); // Basic wildcard search
        let results = sqlx::query_as!(
            Asset,
            r#"
            SELECT
                name, package_name, address, logo, description,
                type_id, category_id, platform_id,
                is_os_verified, is_hidden,
                price, obj.id, rating, downloads, assetlink_sync.domain as website
            FROM obj
            INNER JOIN publishing ON publishing.asset_address = obj.address AND publishing.track_id = 1
            INNER JOIN assetlink_sync ON assetlink_sync.asset_address = obj.address AND assetlink_sync.status = 1
            INNER JOIN build_request ON build_request.asset_address = obj.address AND build_request.status = 1
            INNER JOIN validation_proof ON validation_proof.asset_address = obj.address AND validation_proof.status = 1
            
            WHERE name ILIKE $1
            AND build_request.owner_version = assetlink_sync.owner_version
            AND build_request.owner_version = validation_proof.owner_version
            AND build_request.version_code = publishing.version_code
            AND platform_id = $2
--             AND type_id = $2
            
            ORDER BY downloads DESC
            LIMIT $3 OFFSET $4
            "#,
            search_pattern,
            platform_id,
            // type_id,
            limit,
            offset,
        )
            .fetch_all(self.pool())
            .await?;

        Ok(results)
    }

    pub async fn search_by_category(
        &self,
        term: &str,
        platform_id: i32,
        category_id: i32,
        limit: i64,
        offset: i64,
    ) -> ClientResult<Vec<Asset>> {
        let search_pattern = format!("%{}%", term); // Basic wildcard search
        let results = sqlx::query_as!(
            Asset,
            r#"
            SELECT
                name, package_name, address, logo, description,
                type_id, category_id, platform_id,
                is_os_verified, is_hidden,
                price, obj.id, rating, downloads, assetlink_sync.domain as website
            FROM obj
            INNER JOIN publishing ON publishing.asset_address = obj.address AND publishing.track_id = 1
            INNER JOIN assetlink_sync ON assetlink_sync.asset_address = obj.address AND assetlink_sync.status = 1
            INNER JOIN build_request ON build_request.asset_address = obj.address AND build_request.status = 1
            INNER JOIN validation_proof ON validation_proof.asset_address = obj.address AND validation_proof.status = 1
            
            WHERE name ILIKE $1
            AND build_request.owner_version = assetlink_sync.owner_version
            AND build_request.owner_version = validation_proof.owner_version
            AND build_request.version_code = publishing.version_code
            AND platform_id = $2
            AND category_id = $3
            
            ORDER BY downloads DESC
            LIMIT $4 OFFSET $5
            "#,
            search_pattern,
            platform_id,
            category_id,
            limit,
            offset,
        )
            .fetch_all(self.pool())
            .await?;

        Ok(results)
    }
}

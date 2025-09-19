use crate::data::models::{NewAsset, Asset, RichObject};
use crate::result::ClientResult;
use core_std::empty::Empty;
use db_psql::client::PgClient;
use sqlx::{PgPool, Postgres, Transaction};
use std::ops::Deref;
use hex::ToHex;
use tracing::{error, log};
use codegen_contracts::ext::ToChecksum;
use core_std::hexer;
use crate::data::id::ObjTypeId;

#[derive(Clone)]
pub struct ObjectRepo {
    client: PgClient,
}

impl ObjectRepo {

    pub fn new(client: PgClient) -> Self {
        Self { client }
    }

    pub fn pool(&self) -> &PgPool {
        self.client.pool()
    }

    pub async fn start(
        &self
    ) -> ClientResult<Transaction<'static, Postgres>> {
        let transaction = self.client.start()
            .await?;

        return Ok(transaction);
    }

    pub async fn find_by_id(
        &self,
        id: i64
    ) -> ClientResult<Option<Asset>> {
        let result = sqlx::query_as!(
            Asset,
            r#"
            SELECT
                name, package_name, address, logo, description,
                type_id, category_id, platform_id,
                is_os_verified, is_hidden,
                price, obj.id, rating, downloads, assetlink_sync.domain as website
            FROM obj
                
            INNER JOIN publishing ON publishing.object_address = obj.address AND publishing.track_id = 1 
            INNER JOIN assetlink_sync ON assetlink_sync.object_address = obj.address AND assetlink_sync.status = 1
            INNER JOIN build_request ON build_request.object_address = obj.address AND build_request.status = 1
             
            WHERE build_request.owner_version = assetlink_sync.owner_version
            AND build_request.version_code = publishing.version_code
            AND obj.id = $1
            
            LIMIT 1
            "#,
            id
        )
            .fetch_optional(self.pool())
            .await?;

        return Ok(result)
    }

    pub async fn chart_by_category(
        &self,
        category_id: i32,
        limit: i64,
        offset: i64,
    ) -> ClientResult<Vec<Asset>> {
        let result = sqlx::query_as!(
            Asset,
            r#"
            SELECT
                name, package_name, address, logo, description,
                type_id, category_id, platform_id,
                is_os_verified, is_hidden,
                price, obj.id, rating, downloads, assetlink_sync.domain as website
            FROM obj
            INNER JOIN publishing ON publishing.object_address = obj.address AND publishing.track_id = 1 
            INNER JOIN assetlink_sync ON assetlink_sync.object_address = obj.address AND assetlink_sync.status = 1
            INNER JOIN build_request ON build_request.object_address = obj.address AND build_request.status = 1
             
            WHERE build_request.owner_version = assetlink_sync.owner_version
            AND build_request.version_code = publishing.version_code
            AND category_id = $1
            
            ORDER BY downloads DESC
            LIMIT $2 OFFSET $3
            "#,
            category_id,
            limit,
            offset
        )
            .fetch_all(self.pool())
            .await?;

        return Ok(result)
    }

    pub async fn chart_by_app_type(
        &self,
        platform_id: i32,
        type_id: Option<ObjTypeId>, // TODO split on app/game when it will be many apps
        limit: i64,
        offset: i64,
    ) -> ClientResult<Vec<Asset>> {
        let result = sqlx::query_as!(
            Asset,
            r#"
            SELECT
                name, package_name, address, logo, description,
                type_id, category_id, platform_id,
                is_os_verified, is_hidden,
                price, obj.id, rating, downloads, assetlink_sync.domain as website
            FROM obj
            INNER JOIN publishing ON publishing.object_address = obj.address AND publishing.track_id = 1 
            INNER JOIN assetlink_sync ON assetlink_sync.object_address = obj.address AND assetlink_sync.status = 1
            INNER JOIN build_request ON build_request.object_address = obj.address AND build_request.status = 1
             
            WHERE build_request.owner_version = assetlink_sync.owner_version
            AND build_request.version_code = publishing.version_code
            AND platform_id = $1
--             AND type_id = $2
            
            ORDER BY downloads DESC
            LIMIT $2 OFFSET $3
            "#,
            platform_id,
            // type_id,
            limit,
            offset
        )
            .fetch_all(self.pool())
            .await?;

        return Ok(result)
    }
    
    pub async fn has_by_address(
        &self,
        upper_address: &str,
    ) -> bool {
        let result= sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM obj WHERE address = $1 LIMIT 1;"#,
            upper_address
        )
            .fetch_one(self.pool())
            .await;

        let Ok(result) = result else {
            return false
        };

        return result.unwrap_or(0) > 0;
    }

    pub async fn find_by_address(
        &self,
        address: &str,
    ) -> ClientResult<Option<RichObject>> {
        let result = sqlx::query_as!(
            RichObject,
            r#"
            SELECT
                obj.id, name, package_name, address, logo, description,
                type_id, category_id, platform_id,
                price, rating, downloads, assetlink_sync.domain as website,
                
                is_os_verified,
                COALESCE(assetlink_sync.status = 1, false) AS "is_ownership_verified!: bool",
                COALESCE(build_request.status = 1, false) AS "is_build_verified!: bool"
                
            FROM obj
                
            INNER JOIN publishing ON publishing.object_address = obj.address AND publishing.track_id = 1 
            INNER JOIN assetlink_sync ON assetlink_sync.object_address = obj.address AND assetlink_sync.status = 1
            INNER JOIN build_request ON build_request.object_address = obj.address AND build_request.status = 1
            
            WHERE build_request.version_code = publishing.version_code
            AND build_request.owner_version = assetlink_sync.owner_version
            AND address = $1
            ORDER BY obj.created_at DESC
            
            LIMIT 1
            "#,
            address
        )
            .fetch_optional(self.pool())
            .await?;

        return Ok(result)
    }

    // TODO v2 remove copy
    pub async fn insert_or_update(&self, data: &NewAsset) -> ClientResult<()> {
        let type_id: i32 = data.type_id.clone().into();
        let category_id: i32 = data.category_id.clone().into();
        let platform_id: i32 = data.platform_id.clone().into();
        
        let object = sqlx::query_as!(
            Object,
            r#"
            INSERT INTO obj (
                name, package_name, address, logo, description,
                type_id, category_id, platform_id,
                is_os_verified, is_hidden, price
            )
            
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            
            ON CONFLICT (address) DO UPDATE SET
                name = EXCLUDED.name,
                logo = EXCLUDED.logo,
                description = EXCLUDED.description,
                type_id = EXCLUDED.type_id,
                category_id = EXCLUDED.category_id,
                platform_id = EXCLUDED.platform_id,
                is_os_verified = EXCLUDED.is_os_verified,
                is_hidden = EXCLUDED.is_hidden,
                price = EXCLUDED.price
            "#,
            data.name,
            data.id,
            data.address.upper_checksum(),
            data.logo.clone().or_empty(),
            data.description.clone().or_empty(),
            type_id,
            category_id,
            platform_id,
            data.is_os_verified,
            data.is_hidden,
            data.price
        )
            .execute(self.pool())
            .await?;

        Ok(())
    }

    pub async fn update(&self, data: Asset) -> ClientResult<()> {
        sqlx::query!(
            r#"
            UPDATE obj
            
            SET
                name = $1,
                logo = $2,
                category_id = $3,
                address = $4,
                price = $5,
                type_id = $6,
                description = $7

            WHERE address = $4
            "#,
            data.name,
            data.logo,
            data.category_id,
            data.address.upper_checksum(),
            data.price,
            data.type_id,
            data.description,
        )
            .execute(self.pool())
            .await?;

        return Ok(())
    }

    pub async fn delete(&self, del_id: i64) -> ClientResult<u64> {
        let result = sqlx::query!(
            r#"
             DELETE FROM obj WHERE id = $1
            "#,
            del_id
        )
            .execute(self.pool())
            .await?;

        Ok(result.rows_affected())
    }
    
    pub async fn find_obj_missing_addresses(
        &self,
        addresses: Vec<String>,
    ) -> Vec<String> {
        if addresses.is_empty() {
            return Vec::new()
        }

        let result = sqlx::query_scalar!(
            r#"
            SELECT set_address as "address!"
            
            FROM UNNEST($1::VARCHAR(100)[]) AS input(set_address)
            LEFT JOIN obj ON obj.address = input.set_address
            WHERE obj IS NULL
            "#,
            addresses.deref()
        )
            .fetch_all(self.pool())
            .await;

        match result {
            Ok(result) => result,
            Err(e) => {
                error!("Error: {}", e);
                addresses
            }
        }
    }
}

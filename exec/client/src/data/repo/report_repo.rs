use alloy::primitives::Address;
use crate::data::models::{NewReport, Report};
use crate::result::ClientResult;
use codegen_contracts::ext::ToChecksum;
use db_psql::client::PgClient;
use sqlx::PgPool;

#[derive(Clone)]
pub struct ReportRepo {
    client: PgClient,
}

impl ReportRepo {

    pub fn new(client: PgClient) -> Self {
        Self { client }
    }

    pub fn pool(&self) -> &PgPool {
        self.client.pool()
    }

    pub async fn create(&self, new_report: NewReport) -> ClientResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO report (object_address, email, category_id, subcategory_id, description)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            new_report.object_address.upper_checksum(),
            new_report.email,
            new_report.category_id,
            new_report.subcategory_id,
            new_report.description
        )
            .execute(self.pool())
            .await?;

        return Ok(())
    }

    pub async fn find_by_id(&self, report_id: i64) -> ClientResult<Option<Report>> {
        let result = sqlx::query_as!(
            Report,
            r#"
            SELECT id, object_address, email, category_id, subcategory_id, description
            FROM report WHERE id = $1
            "#,
            report_id
        )
            .fetch_optional(self.pool())
            .await?;

        return Ok(result)
    }

    // Find reports for a specific object (leveraging index)
    pub async fn find_by_object_id(
        &self,
        object_address: Address,
        limit: i64,
        offset: i64,
    ) -> ClientResult<Vec<Report>> {
        let result = sqlx::query_as!(
            Report,
            r#"
            SELECT id, object_address, email, category_id, subcategory_id, description
            FROM report
            WHERE object_address = $1
            LIMIT $2 OFFSET $3
            "#,
            object_address.upper_checksum(),
            limit,
            offset
        )
            .fetch_all(self.pool())
            .await?;

        return Ok(result)
    }

    // Find reports by email (leveraging potential index - idx_reports_user_id might be on email?)
    // NOTE: The schema has an index named idx_reports_user_id, but no user_id column.
    // Assuming the index is actually on `email` or should be.
    pub async fn find_by_email(
        &self,
        email: &str,
        limit: i64,
        offset: i64,
    ) -> ClientResult<Vec<Report>> {
        let result = sqlx::query_as!(
            Report,
            r#"
            SELECT id, object_address, email, category_id, subcategory_id, description
            FROM report
            WHERE email = $1
            LIMIT $2 OFFSET $3
            "#,
            email,
            limit,
            offset
        )
            .fetch_all(self.pool())
            .await?;

        return Ok(result)
    }

    // Reports usually aren't updated or deleted via API, but you could add delete if needed.
    pub async fn delete(&self, report_id: i64) -> ClientResult<u64> {
        let result = sqlx::query!(
            r#"
            DELETE FROM report WHERE id = $1
            "#,
            report_id
        )
            .execute(self.pool())
            .await?;

        Ok(result.rows_affected())
    }
}
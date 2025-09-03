use std::ops::{Range, RangeInclusive};
use crate::data::models::{NewReview, Review};
use crate::result::{ClientError, ClientResult};
use db_psql::client::PgClient;
use sqlx::PgPool;

#[derive(Clone)]
pub struct ReviewRepo {
    client: PgClient,
}

const RATING_RAGE: RangeInclusive<i32> = 1..=5;

impl ReviewRepo {

    pub fn new(client: PgClient) -> Self {
        Self { client }
    }

    pub fn pool(&self) -> &PgPool {
        self.client.pool()
    }

    pub async fn create(&self, new_review: NewReview) -> ClientResult<()> {
        if !RATING_RAGE.contains(&new_review.rating) {
            return Err(ClientError::Conflict("Rating must be between 1 and 5".to_string()));
        }

        sqlx::query!(
            r#"
            INSERT INTO review (object_id, user_id, rating, text)
            VALUES ($1, $2, $3, $4)
            "#,
            new_review.object_id,
            new_review.user_id,
            new_review.rating,
            new_review.text
        )
            .execute(self.pool())
            .await?;

        return Ok(())
    }

    pub async fn find_by_id(&self, review_id: i64) -> ClientResult<Option<Review>> {
        let result = sqlx::query_as!(
            Review,
            r#"
            SELECT id, object_id, user_id, rating, text
            FROM review WHERE id = $1
            "#,
            review_id
        )
            .fetch_optional(self.pool())
            .await?;

        return Ok(result)
    }

    // Find reviews for a specific object (leveraging index)
    pub async fn find_by_object_id(
        &self,
        object_id: i64,
        limit: i64,
        offset: i64,
    ) -> ClientResult<Vec<Review>> {
        let result = sqlx::query_as!(
            Review,
            r#"
            SELECT id, object_id, user_id, rating, text
            FROM review
            WHERE object_id = $1
            LIMIT $2 OFFSET $3
            "#,
            object_id,
            limit,
            offset
        )
            .fetch_all(self.pool())
            .await?;

        return Ok(result)
    }

    // Find reviews by a specific user (leveraging index)
    pub async fn find_by_user_id(
        &self,
        user_id: &str,
        limit: i64,
        offset: i64,
    ) -> ClientResult<Vec<Review>> {
        let result = sqlx::query_as!(
            Review,
            r#"
            SELECT id, object_id, user_id, rating, text
            FROM review
            WHERE user_id = $1
            LIMIT $2 OFFSET $3
            "#,
            user_id,
            limit,
            offset
        )
            .fetch_all(self.pool())
            .await?;

        return Ok(result)
    }

    pub async fn delete(&self, review_id: i64) -> ClientResult<u64> {
        let result = sqlx::query!(
            r#"
            DELETE FROM review WHERE id = $1
            "#,
            review_id
        )
            .execute(self.pool())
            .await?;

        Ok(result.rows_affected())
    }

    pub async fn delete_by_user_object(&self, usr_id: &str, obj_id: i64) -> ClientResult<u64> {
        let result = sqlx::query!(
            r#"
            DELETE FROM review WHERE user_id = $1 AND object_id = $2
            "#,
            usr_id,
            obj_id
        )
            .execute(self.pool())
            .await?;

        Ok(result.rows_affected())
    }
}
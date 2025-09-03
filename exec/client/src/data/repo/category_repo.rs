use sqlx::PgPool;
use db_psql::client::PgClient;
use crate::data::models::{Category, NewCategory};
use crate::result::ClientResult;

#[derive(Clone)]
pub struct CategoryRepo {
    client: PgClient,
}

impl CategoryRepo {

    pub fn new(client: PgClient) -> Self {
        Self { client }
    }

    pub fn pool(&self) -> &PgPool {
        self.client.pool()
    }

    pub async fn put_all(&self, categories: Vec<NewCategory>) -> ClientResult<()> {
        // sqlx::query_as!(
        //     Category,
        //     r#"
        //     INSERT INTO category (id, name, type_id)
        //     VALUES ($1, $2, $3)
        //     "#,
        //     new_category.id
        //     new_category.name
        //     new_category.type_id
        // )
        //     .fetch_one(self.pool())
        //     .await?

        return Ok(())
    }

    pub async fn get_all(&self) -> ClientResult<Vec<Category>> {
        let result = sqlx::query_as!(
            Category,
            " SELECT id, type_id, name FROM category ORDER BY name"
        )
            .fetch_all(self.pool())
            .await?;

        Ok(result)
    }
}

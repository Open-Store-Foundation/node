use sqlx::PgPool;
use db_psql::client::PgClient;
use net_client::node::result::EthResult;

#[derive(Clone)]
pub struct ErrorRepo {
    client: PgClient,
}

impl ErrorRepo {

    pub fn new(client: PgClient) -> Self {
        Self { client }
    }

    pub fn pool(&self) -> &PgPool {
        self.client.pool()
    }

    pub async fn insert_fatal_tx(&self, hash: String) -> EthResult<()> {
        return Ok(())
    }
    
    pub async fn insert_error_tx(&self, hash: String) -> EthResult<()> {
        return Ok(())
    }
}

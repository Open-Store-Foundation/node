use redis::RedisResult;

#[derive(Clone, Debug)]
pub struct RedisClient {
    client: redis::Client,
}

impl RedisClient {
    pub fn new(redis_url: String) -> RedisResult<Self> {
        let client = redis::Client::open(redis_url)?;
        Ok(Self { client })
    }

    pub async fn get_connection(&self) -> RedisResult<redis::aio::MultiplexedConnection> {
        return self.client
            .get_multiplexed_tokio_connection()
            .await
    }
}
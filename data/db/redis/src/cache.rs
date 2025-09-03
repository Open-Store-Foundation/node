use redis::{AsyncCommands, RedisError};
use serde::de::DeserializeOwned;
use serde::Serialize;
use thiserror::Error;
use tracing::{debug, trace};
use crate::client::RedisClient;

#[derive(Clone, Debug)]
pub struct RedisCache {
    client: RedisClient
}

pub type KeyValueResult<T> = Result<T, KeyValueError>;

#[derive(Error, Debug)]
pub enum KeyValueError {
    #[error("Redis error: {0}")]
    Redis(#[from] RedisError),
    #[error("Json error: {0}")]
    Json(#[from] serde_json::Error),
}

impl RedisCache {

    pub fn new(client: RedisClient) -> Self {
        Self { client }
    }

    pub async fn set_str(
        &self,
        key: &str,
        value: &str,
        ttl_seconds: Option<u64>,
    ) -> KeyValueResult<()> {
        debug!("Setting key: {} with TTL: {:?}", key, ttl_seconds);

        let mut conn = self.client.get_connection()
            .await?;

        match ttl_seconds {
            Some(ttl) => {
                conn.set_ex::<_, _, ()>(key, value, ttl)
                    .await?;
                trace!("Executed SETEX for key: {}", key);
            }
            None => {
                conn.set::<_, _, ()>(key, value)
                    .await?;
                trace!("Executed SET for key: {}", key);
            }
        }

        Ok(())
    }

    pub async fn set<V : Serialize>(
        &self,
        key: &str,
        value: &V,
        ttl_seconds: Option<u64>,
    ) -> KeyValueResult<()> {
        let serialized_value = serde_json::to_string(value)?;

        self.set_str(key, serialized_value.as_ref(), ttl_seconds)
            .await?;

        Ok(())
    }

    pub async fn get_str(&self, key: &str) -> KeyValueResult<Option<String>> {
        let mut conn = self.client.get_connection()
            .await?;

        let result: Option<String> = conn.get(key).
            await?;

        match result {
            Some(value) => {
                Ok(Some(value))
            }
            None => {
                trace!("Key not found: {}", key);
                Ok(None)
            }
        }
    }

    pub async fn get<V>(&self, key: &str) -> KeyValueResult<Option<V>> where V : DeserializeOwned {
        match self.get_str(key).await? {
            Some(serialized_value) => {
                trace!("Found key: {}, raw value: {}", key, serialized_value);
                let value = serde_json::from_str(&serialized_value)?;
                Ok(Some(value))
            }
            None => {
                trace!("Key not found: {}", key);
                Ok(None)
            }
        }
    }

    pub async fn delete(&self, key: &str) -> KeyValueResult<bool> {
        debug!("Deleting key: {}", key);

        let mut conn = self.client.get_connection()
            .await?;

        let deleted_count: i64 = conn.del(key)
            .await?;

        Ok(deleted_count > 0)
    }
}

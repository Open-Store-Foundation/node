use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use base64::Engine;
use db_redis::cache::{KeyValueResult, RedisCache};
use std::sync::Arc;

pub struct CacheRepo {
    cache: Arc<RedisCache>
}

const ETAG_TTL: u64 = 24 * 60 * 60; // TODO to config

impl CacheRepo {

    pub fn new(cache: Arc<RedisCache>) -> Self {
        Self { cache }
    }

    pub async fn get_etag(&self, key: &str) -> Option<String> {
        return self.cache.get_str(key)
            .await
            .unwrap_or(None)
    }

    pub async fn set_etag(&self, key: &str, etag: &str) -> KeyValueResult<()> {
        return self.cache.set_str(key, etag, Some(ETAG_TTL))
            .await
    }

    pub async fn set_etag_by_content(&self, key: &str, content: &str) -> KeyValueResult<String> {
        let result = blake3::hash(content.as_bytes());
        let new_etag = format!(r#"W/"{}""#, BASE64_URL_SAFE_NO_PAD.encode(result.as_bytes()));

        self.set_etag(key.as_ref(), new_etag.as_ref())
            .await?;

        return Ok(new_etag);
    }

    pub async fn set_etag_with_content(&self, key: &str, content: &str, ttl: Option<u64>) -> KeyValueResult<String> {
        let result = blake3::hash(content.as_bytes());
        let new_etag = format!(r#"W/"{}""#, BASE64_URL_SAFE_NO_PAD.encode(result.as_bytes()));

        self.cache.set_str(new_etag.as_ref(), content, ttl)
            .await?;

        self.set_etag(key.as_ref(), new_etag.as_ref())
            .await?;

        return Ok(new_etag);
    }

    pub async fn get_content(&self, key: &str) -> Option<String> {
        return self.cache.get_str(key)
            .await
            .unwrap_or(None)
    }
}

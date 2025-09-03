use crate::data::repo::cache_repo::CacheRepo;
use crate::result::{ClientError, ClientResult};
use axum::http::StatusCode;
use axum::response::Response;
use bytes::Bytes;
use core_std::empty::Empty;
use headers::{ETag, IfNoneMatch};
use net_result::{response_data, ResponseConstructor};
use serde::Serialize;
use std::future::Future;
use std::str::FromStr;
use std::sync::Arc;
use tracing::debug;
// https://gnfd-testnet-sp3.bnbchain.org/view/test-andrew/bla.apk
pub struct EtagHandler {
    cache_repo: Arc<CacheRepo>
}

impl EtagHandler {

    pub fn new(cache_repo: Arc<CacheRepo>) -> Self {
        Self { cache_repo }
    }

    // TODO move to admin and calculate etag
    pub async fn etag_cache_or_static(
        &self,
        etag_key: String,
        none_match: Option<IfNoneMatch>,
        cache_key: String,
    ) -> ClientResult<Response> {
        let actual_etag = self.cache_repo.get_etag(etag_key.as_ref())
            .await
            .or_empty();

        if self.compare_etag(none_match, actual_etag.as_ref()) {
            debug!("etag not-modified");
            return Ok(
                Response::with_code(StatusCode::NOT_MODIFIED)?
            );
        }
        
        let content = self.cache_repo.get_content(cache_key.as_ref())
            .await
            .ok_or(ClientError::NotFound)?;

        let result = self.cache_repo.set_etag_by_content(
            etag_key.as_ref(), content.as_ref()
        ).await;

        debug!("etag cache-miss");
        return Ok(
            match result {
                Ok(new_etag) => Response::with_etag(new_etag, Bytes::from_owner(content))?,
                Err(_) => Response::with_body(Bytes::from_owner(content))?
            }
        );
    }
    
    pub async fn etag_cache_or<F, Fut, R>(
        &self,
        etag_key: String,
        none_match: Option<IfNoneMatch>,
        ttl: Option<u64>,
        producer: F,
    ) -> ClientResult<Response>
    where
        F: FnOnce() -> Fut,
        R: Serialize,
        Fut: Future<Output=ClientResult<R>> {
        let actual_etag = self.cache_repo.get_etag(etag_key.as_ref())
            .await
            .or_empty();

        if self.compare_etag(none_match, actual_etag.as_ref()) {
            debug!("etag not-modified");
            return Ok(
                Response::with_code(StatusCode::NOT_MODIFIED)?
            );
        }

        if actual_etag.len() > 0 {
            if let Some(result) = self.cache_repo.get_content(actual_etag.as_ref()).await {
                debug!("etag cache-hit");
                return Ok(
                    Response::with_etag(actual_etag, Bytes::from_owner(result))?
                )
            }
        }

        let result = producer()
            .await?;

        let response = response_data(result);
        let content = serde_json::to_string(&response.data)?;

        let result = self.cache_repo.set_etag_with_content(
            etag_key.as_ref(), content.as_ref(), ttl
        ).await;

        debug!("etag cache-miss");
        return Ok(
            match result {
                Ok(new_etag) => Response::with_etag(new_etag, Bytes::from_owner(content))?,
                Err(_) => Response::with_body(Bytes::from_owner(content))?
            }
        );
    }

    fn compare_etag(&self, user_etag: Option<IfNoneMatch>, remote_etag: &str) -> bool {
        let Some(none_match) = user_etag else {
            return false;
        };

        if remote_etag.len() == 0 {
            return false;
        }

        let Ok(etag) = ETag::from_str(remote_etag) else {
            return false;
        };

        return !none_match.precondition_passes(&etag)
    }
}


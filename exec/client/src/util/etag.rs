use crate::result::{ClientError, ClientResult};
use axum::http::HeaderMap;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use headers::{ETag, HeaderMapExt, IfNoneMatch};
use openssl::sha::sha256;
use serde::Serialize;
use std::str::FromStr;
use tracing::error;

// // Generates a SHA-256 hash of the JSON representation of the data,
// // Base64 encoded for the ETag header.
// pub fn generate_etag<T: Serialize>(data: &T) -> Result<String, serde_json::Error> {
//     let data_bytes = serde_json::to_vec(data)?;
//     let hash = sha256(data_bytes.as_slice());
//     // Wrap in quotes for weak ETag, or omit for strong
//     return Ok(format!(r#"W/"{}""#, URL_SAFE_NO_PAD.encode(hash)))
// }
//
// // Checks if the If-None-Match header matches the generated ETag
// pub fn check_if_none_match<T: Serialize>(
//     headers: &HeaderMap,
//     etag: &String,
// ) -> ClientResult<bool> {
//     // let generated_etag = ETag::from_str(&generate_etag(data)?).map_err(|e| {
//     //     error!("Failed to parse generated ETag: {}", e);
//     //     ClientError::Config("Internal ETag generation error".into()) // Should not happen
//     // })?;
//
//     return if let Some(if_none_match) = headers.typed_get::<IfNoneMatch>() {
//         if if_none_match.precondition_passes(&etag) {
//             Ok(true) // ETag matches, client cache is valid
//         } else {
//             Ok(false) // ETag doesn't match
//         }
//     } else {
//         Ok(false) // No If-None-Match header present
//     }
// }
//
// // Helper to create ETag header value
// pub fn create_etag_header(etag_str: String) -> ClientResult<ETag> {
//     return ETag::from_str(&etag_str)
//         .map_err(|e| {
//             error!("Failed to parse generated ETag for header: {}", e);
//             ClientError::Config("Internal ETag header error".into())
//         })
// }

use axum::http::{header, HeaderMap};
use tracing::{info, instrument, warn};

const PROTOBUF_CONTENT_TYPES: &[&str] = &[
    "application/protobuf",
    "application/x-protobuf",
];

pub fn check_protobuf_content_type(headers: &HeaderMap) -> bool {
    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|val| val.to_str().ok());

    let Some(ct) = content_type else {
        warn!("content-type is empty!");
        return false;
    };

    let has = PROTOBUF_CONTENT_TYPES.iter()
        .any(|&allowed_ct| {
            ct.eq_ignore_ascii_case(allowed_ct)
        });

    if !has {
        warn!("wrong content type: {}!", ct);
    }

    return has;
}

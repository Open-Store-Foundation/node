use crate::data::stat_buffer::StatBuffer;
use crate::result::{StatError, StatResult};
use crate::AppState;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::IntoResponse;
use net_result::{response_null, response_nullable, response_ok, ResponseData};
use std::sync::Arc;
use tracing::{instrument, warn};

pub async fn create_event(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> StatResult<impl IntoResponse> {
    // if check_protobuf_content_type(&headers) { // TODO v2 add protobuf
    //     return Err(StatError::InvalidContentType);
    // }

    if body.is_empty() {
        warn!("received empty request body, skipping Kafka send.");
        return Err(StatError::EmptyBody);
    }

    match state.state_repo.publish(body.to_vec()).await {
        Ok(_) => Ok((StatusCode::ACCEPTED, response_null::<String>())),
        Err(kafka_err) => Err(StatError::KafkaProduceError(kafka_err))
    }
}

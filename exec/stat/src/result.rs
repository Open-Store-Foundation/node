use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use thiserror::Error;
use tracing::error;
use net_result::ResponseDataError;
use prost::DecodeError;

pub type StatResult<T> = Result<T, StatError>;

#[derive(Error, Debug)]
pub enum StatError {
    #[error("Failed to queue event")]
    KafkaProduceError(#[from] rdkafka::error::KafkaError),
    #[error("Invalid Content-Type. Expected application/protobuf")]
    InvalidContentType,
    #[error("Failed to read request body")]
    BodyReadError(axum::Error),
    #[error("Body is empty")]
    EmptyBody,
    #[error("Failed to decode protobuf message")]
    DecodeError(#[from] DecodeError),
}

pub enum StatErrorCodes {
    Unknown = 0,
}

impl IntoResponse for StatError {
    fn into_response(self) -> Response {
        let message = self.to_string();
        let (status, code, message) = match self {
            StatError::KafkaProduceError(err) => {
                error!(error = %err, "Failed to produce message to Kafka");
                (StatusCode::INTERNAL_SERVER_ERROR, StatErrorCodes::Unknown, message)
            }

            StatError::EmptyBody => {
                (StatusCode::BAD_REQUEST, StatErrorCodes::Unknown, message)
            }

            StatError::InvalidContentType => {
                (StatusCode::UNSUPPORTED_MEDIA_TYPE, StatErrorCodes::Unknown, message)
            }

            StatError::BodyReadError(err) => {
                error!(error = %err, "Failed to read request body");
                (StatusCode::BAD_REQUEST, StatErrorCodes::Unknown, message)
            }

            StatError::DecodeError(err) => {
                error!(error = %err, "Failed to decode protobuf message");
                (StatusCode::BAD_REQUEST, StatErrorCodes::Unknown, message)
            }
        };

        let error = ResponseDataError { code: code as i32, message };

        (status, Json(error)).into_response()
    }
}

use axum::body::Body;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use bytes::Bytes;
use derive_more::Display;
use serde::Serialize;

#[derive(Debug, Clone, Display)]
pub struct ResponseOk<T> where T : Serialize {
    pub data: ResponseData<T>,
}

#[derive(Debug, Clone, Display, Serialize)]
pub struct ResponseData<T> where T : Serialize {
    pub data: T,
}

#[derive(Debug, Clone, Display)]
pub struct ResponseErr {
    pub data: ResponseDataError,
}

#[derive(Debug, Clone, Display, Serialize)]
#[display("[ code: {code}, message: {message} ]")]
pub struct ResponseDataError {
    pub code: i32,
    pub message: String,
}

pub fn response_err(code: i32, message: String) -> ResponseErr {
    return ResponseErr { data: ResponseDataError { code, message } }
}

pub fn response_ok() -> ResponseOk<String> {
    return ResponseOk { data: ResponseData { data: "Ok".to_string() } }
}

pub fn response_null<T>() -> ResponseOk<Option<T>> where T : Serialize {
    return ResponseOk { data: ResponseData { data: None } }
}

pub fn response_nullable() -> ResponseOk<Option<String>> {
    return ResponseOk { data: ResponseData { data: None } }
}

pub fn response_empty<T>() -> ResponseOk<Vec<T>> where T : Serialize {
    return ResponseOk { data: ResponseData { data: vec![] } }
}

pub fn response_data<T>(data: T) -> ResponseOk<T> where T : Serialize {
    return ResponseOk { data: ResponseData { data } }
}

impl <T> IntoResponse for ResponseOk<T> where T : Serialize {
    fn into_response(self) -> Response {
        Json(self.data).into_response()
    }
}

impl IntoResponse for ResponseErr {
    fn into_response(self) -> Response {
        Json(self.data).into_response()
    }
}

pub trait ResponseConstructor {
    fn with_code(
        status_code: StatusCode
    ) -> axum::http::Result<Response>;

    fn with_body(
        body: Bytes
    ) -> axum::http::Result<Response>;

    fn with_etag(
        etag: String,
        body: Bytes
    ) -> axum::http::Result<Response>;
}

impl ResponseConstructor for Response {
    fn with_code(status_code: StatusCode) -> axum::http::Result<Response> {
        return Response::builder()
            .header(header::CONTENT_TYPE, "application/json")
            .status(status_code)
            .body(Body::empty());
    }

    fn with_body(
        body: Bytes
    ) -> axum::http::Result<Response> {
        return Response::builder()
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body))
    }

    fn with_etag(
        etag: String,
        body: Bytes
    ) -> axum::http::Result<Response> {
        return Response::builder()
            .header(header::ETAG, etag)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body))
    }
}

use std::process::Output;

use axum::body::Full;
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{body, BoxError};

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{0}")]
    IoError(#[from] std::io::Error),
    #[error("{0} should be present after trunk build but is not")]
    BuildFileNotFound(&'static str),
    #[error("request must have a body but none was found")]
    NoBody,
    #[error("build failed with error {}\n{}", .0.status, String::from_utf8_lossy(&.0.stderr))]
    BuildFailed(Output),
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
    #[error("failed to deserialize bson: {0}")]
    BsonDeserializeError(#[from] bson::de::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self {
            ApiError::IoError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::BuildFileNotFound(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::NoBody => StatusCode::BAD_REQUEST,
            ApiError::BuildFailed(_) => StatusCode::BAD_REQUEST,
            ApiError::Unknown(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::BsonDeserializeError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        Response::builder()
            .status(status)
            .header(
                header::CONTENT_TYPE,
                HeaderValue::from_static(mime::TEXT_PLAIN_UTF_8.as_ref()),
            )
            .body(body::boxed(Full::from(self.to_string())))
            .unwrap()
    }
}

pub async fn timeout_or_500(err: BoxError) -> (StatusCode, String) {
    if err.is::<tower::timeout::error::Elapsed>() {
        (
            StatusCode::REQUEST_TIMEOUT,
            "Request took too long".to_string(),
        )
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Unhandled internal error: {}", err),
        )
    }
}

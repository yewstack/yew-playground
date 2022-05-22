use std::process::Output;

use axum::body;
use axum::body::Full;
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{0}")]
    IoError(#[from] std::io::Error),
    #[error("{0} should be present after trunk build but is not")]
    BuildFileNotFound(&'static str),
    #[error("build failed with error {}\n{}", .0.status, String::from_utf8_lossy(&.0.stderr))]
    BuildFailed(Output),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self {
            ApiError::IoError(_) => (StatusCode::INTERNAL_SERVER_ERROR),
            ApiError::BuildFileNotFound(_) => (StatusCode::INTERNAL_SERVER_ERROR),
            ApiError::BuildFailed(_) => (StatusCode::BAD_REQUEST),
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

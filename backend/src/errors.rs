use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{0}")]
    IoError(std::io::Error),
    #[error("request must have a body but none was found")]
    NoBody,
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self {
            ApiError::IoError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::NoBody => StatusCode::BAD_REQUEST,
            ApiError::Unknown(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, self.to_string()).into_response()
    }
}

pub async fn timeout_or_500(err: axum::BoxError) -> (StatusCode, String) {
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

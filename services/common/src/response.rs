use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::Serialize;

pub struct Bson<T>(pub T);
impl<T> IntoResponse for Bson<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        let bytes = match bson::to_vec(&self.0) {
            Ok(res) => res,
            Err(err) => {
                return (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response();
            }
        };

        let mut res = Response::new(axum::body::Body::from(bytes));
        res.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/bson"),
        );
        res
    }
}

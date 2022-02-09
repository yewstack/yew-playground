use axum::{
    body::{self, Full},
    response::{IntoResponse, Response},
};
use axum::http::{header, HeaderValue, StatusCode};
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
                return Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header(
                        header::CONTENT_TYPE,
                        HeaderValue::from_static(mime::TEXT_PLAIN_UTF_8.as_ref()),
                    )
                    .body(body::boxed(Full::from(err.to_string())))
                    .unwrap();
            }
        };

        let mut res = Response::new(body::boxed(Full::from(bytes)));
        res.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/bson"),
        );
        res
    }
}

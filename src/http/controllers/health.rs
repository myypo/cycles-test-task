use axum::http::StatusCode;

use crate::http::controllers::{response, Response};

pub async fn check_health() -> Response<()> {
    response(StatusCode::OK, ())
}

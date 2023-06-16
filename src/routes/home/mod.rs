use axum::{body::Full, response::Response};
use axum_macros::debug_handler;
use hyper::body::Bytes;

#[debug_handler]
pub async fn home() -> Response<Full<Bytes>> {
    Response::builder()
        .header("Content-Type", "text/html; charset=utf-8")
        .body(Full::from(include_str!("home.html")))
        .unwrap()
}

use axum::{response::IntoResponse, Json};
use axum_macros::debug_handler;
use hyper::StatusCode;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct BodyData {
  title: String,
  content: Content,
}

#[derive(Deserialize)]
pub struct Content {
  text: String,
  html: String,
}

#[debug_handler]
pub async fn publish_newsletter(_body: Json<BodyData>) -> impl IntoResponse {
  StatusCode::OK
}
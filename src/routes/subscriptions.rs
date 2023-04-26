use axum::{response::IntoResponse, Form};
use hyper::StatusCode;
use serde::Deserialize;
use tracing::info;

// region: -- Subscribe Handler
#[derive(Deserialize, Debug)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn handler_subscribe(Form(data): Form<FormData>) -> impl IntoResponse {
    info!("{:<8} - handler_subscribe - {data:?}", "HANDLER");

    StatusCode::OK
}
// endregion: -- Subscribe Handler

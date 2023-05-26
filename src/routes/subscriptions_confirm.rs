use crate::error::Result;
use axum::{extract::Query, http::StatusCode, response::IntoResponse};

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(_parameters))]
pub async fn handler_confirm(Query(_parameters): Query<Parameters>) -> Result<impl IntoResponse> {
    Ok(StatusCode::OK)
}

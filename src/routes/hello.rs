use axum::{
    extract::Query,
    response::{Html, IntoResponse},
};
use serde::Deserialize;
use tracing::info;

// region: -- Hello Handlers
#[derive(Debug, Deserialize)]
pub struct HelloParams {
    name: Option<String>,
}

// e.g, /hello?name=John
pub async fn handler_hello(Query(params): Query<HelloParams>) -> impl IntoResponse {
    info!("{:<8} - handler_hello - {params:?}", "HANDLER");

    let name = params.name.as_deref().unwrap_or("Frank");
    Html(format!("Hello <strong>{name}</strong>"))
}
// endregion: -- Hello Handlers

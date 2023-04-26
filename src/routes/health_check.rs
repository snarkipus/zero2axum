use axum::response::IntoResponse;
use hyper::StatusCode;
use tracing::info;

// region: -- Health Check Handler
pub async fn handler_health_check() -> impl IntoResponse {
    info!("{:<8} - handler_health_check", "HANDLER");

    StatusCode::OK
}
// endregion: -- Health Check Handler

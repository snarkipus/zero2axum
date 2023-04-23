#![allow(unused)]
use axum::{
    extract::Query,
    response::{Html, IntoResponse},
    routing::get,
    Router, http::StatusCode,
};
use serde::Deserialize;
use std::{net::SocketAddr, str::FromStr};
use tracing::{info, warn, Level};
use tracing_subscriber::{filter::Targets, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    // region: -- Tracing: Initialize
    let filter = Targets::from_str(std::env::var("RUST_LOG").as_deref().unwrap_or("info"))
        .expect("RUST_LOG must be a valid tracing filter");
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .pretty()
        // .json() // Uncomment to enable JSON logging
        .finish()
        .with(filter)
        .init();
    // endregion: --- Tracing: Initialize

    // region: -- Sentry.io error reporting
    let _guard = sentry::init((
        std::env::var("SENTRY_DSN").expect("$SENTRY_DSN must be set"),
        sentry::ClientOptions {
            release: sentry::release_name!(),
            ..Default::default()
        },
    ));
    // endregion: --- Sentry.io error reporting

    let app = Router::new()
        .route("/", get(handler_hello))
        .route("/health_check", get(handler_health_check));

    let quit_sig = async {
        tokio::signal::ctrl_c().await;
        warn!("Received Ctrl-C, shutting down gracefully...");
    };

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Listening on http://{addr}");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(quit_sig)
        .await
        .unwrap();

    Ok(())
}

#[derive(Debug, Deserialize)]
struct HelloParams {
    name: Option<String>,
}

// e.g, /hello?name=John
async fn handler_hello(Query(params): Query<HelloParams>) -> impl IntoResponse {
    info!("{:<8} - handler_hello - {params:?}", "HANDLER");

    let name = params.name.as_deref().unwrap_or("Frank");
    Html(format!("Hello <strong>{name}</strong>"))
}

async fn handler_health_check() -> impl IntoResponse {
    info!("{:<8} - handler_health_check", "HANDLER");

    StatusCode::OK
}

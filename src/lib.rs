// #![allow(unused)]
use axum::{
    extract::Query,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, IntoMakeService},
    Router, Server,
};
use hyper::{server::conn::AddrIncoming, Body};
use serde::Deserialize;
use std::{net::TcpListener, str::FromStr};
use tracing::{info, Level};
use tracing_subscriber::{filter::Targets, layer::SubscriberExt, util::SubscriberInitExt};

async fn handler_health_check() -> impl IntoResponse {
    info!("{:<8} - handler_health_check", "HANDLER");

    StatusCode::OK
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

pub fn run(
    listener: TcpListener,
) -> Result<Server<AddrIncoming, IntoMakeService<Router<(), Body>>>, std::io::Error> {
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

    let server = axum::Server::from_tcp(listener)
        .unwrap_or_else(|e| {
            panic!("Failed to bind random port: {}", e);
        })
        .serve(app.into_make_service());
    Ok(server)
}

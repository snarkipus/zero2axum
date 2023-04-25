#![allow(unused)]
use axum::{
    extract::{Form, Query},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post, IntoMakeService},
    Router, Server,
};
use hyper::{server::conn::AddrIncoming, Body};
use serde::Deserialize;
use std::net::TcpListener;
use tracing::info;

// region: -- Health Check Handler
async fn handler_health_check() -> impl IntoResponse {
    info!("{:<8} - handler_health_check", "HANDLER");

    StatusCode::OK
}
// endregion: -- Health Check Handler

#[derive(Deserialize, Debug)]
struct FormData {
    email: String,
    name: String,
}

async fn handler_subscribe(Form(data): Form<FormData>) -> impl IntoResponse {
    info!("{:<8} - handler_subscribe - {data:?}", "HANDLER");

    StatusCode::OK
}

// region: -- Hello Handlers
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
// endregion: -- Hello Handlers

pub fn run(
    listener: TcpListener,
) -> Result<Server<AddrIncoming, IntoMakeService<Router<(), Body>>>, std::io::Error> {
    let app = Router::new()
        .route("/", get(handler_hello))
        .route("/health_check", get(handler_health_check))
        .route("/subscribe", post(handler_subscribe));

    let server = axum::Server::from_tcp(listener)
        .unwrap_or_else(|e| {
            panic!("Failed to bind random port: {}", e);
        })
        .serve(app.into_make_service());
    Ok(server)
}

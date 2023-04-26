use std::net::TcpListener;

use axum::{
    routing::{get, post, IntoMakeService},
    Router, Server,
};
use hyper::{server::conn::AddrIncoming, Body};

use crate::routes::{handler_health_check, handler_hello, handler_subscribe};

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

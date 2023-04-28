use std::net::TcpListener;

use axum::{
    routing::{get, post, IntoMakeService},
    Router, Server,
};
use hyper::{server::conn::AddrIncoming, Body};
use surrealdb::{engine::remote::ws::Client, Surreal};

use crate::routes;

pub async fn run(
    listener: TcpListener,
    db: Surreal<Client>,
) -> Result<Server<AddrIncoming, IntoMakeService<Router<(), Body>>>, std::io::Error> {
    let app = Router::new()
        .route("/", get(routes::handler_hello))
        .route("/health_check", get(routes::handler_health_check))
        .route("/subscribe", post(routes::handler_subscribe))
        .with_state(db);

    let server = Server::from_tcp(listener)
        .unwrap_or_else(|e| {
            panic!("Failed to bind random port: {}", e);
        })
        .serve(app.into_make_service());
    Ok(server)
}

use std::net::TcpListener;

use axum::{
    middleware,
    response::{IntoResponse, Response},
    routing::{get, post, IntoMakeService},
    Json, Router, Server,
};
use hyper::{server::conn::AddrIncoming, Body, Method, Uri};
use serde_json::json;
use surrealdb::{engine::remote::ws::Client, Surreal};
use tracing::info;
use uuid::Uuid;

use crate::{error, routes};

pub async fn run(
    listener: TcpListener,
    db: Surreal<Client>,
) -> Result<Server<AddrIncoming, IntoMakeService<Router<(), Body>>>, std::io::Error> {
    let app = Router::new()
        .route("/", get(routes::handler_hello))
        .route("/health_check", get(routes::handler_health_check))
        .layer(middleware::map_response(main_response_mapper))
        .route("/subscribe", post(routes::handler_subscribe))
        .with_state(db);

    let server = Server::from_tcp(listener)
        .unwrap_or_else(|e| {
            panic!("Failed to bind random port: {}", e);
        })
        .serve(app.into_make_service());
    Ok(server)
}

async fn main_response_mapper(
    // db: Option<Surreal<Client>>,
    _uri: Uri,
    _req_method: Method,
    res: Response,
) -> Response {
    info!("->> {:<12} - main_response_mapper", "RES_MAPPER");
    let uuid = Uuid::new_v4();

    // Get the response error
    let service_error = res.extensions().get::<error::Error>();
    let client_status_error = service_error.map(|e| e.client_status_and_error());

    // If client error, build new response
    let error_response = client_status_error
        .as_ref()
        .map(|(status_code, client_error)| {
            let client_error_body = json!({
                "error": {
                    "type": client_error.as_ref(),
                    "req_uuid": uuid.to_string(),
                }
            });

            println!("  ->> client_error_body: {client_error_body}");

            // Build the response
            (*status_code, Json(client_error_body)).into_response()
        });

    // Build and log the server log
    let _client_error = client_status_error.unzip().1;
    // log_request(uuid, req_method, uri, ctx, service_error, client_error).await;

    println!();
    error_response.unwrap_or(res)
}

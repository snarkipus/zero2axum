#[allow(unused_imports)]
use axum::{
    extract::Extension,
    middleware,
    response::{IntoResponse, Response},
    routing::{get, post, IntoMakeService},
    Json, Router, Server,
};

use hyper::{server::conn::AddrIncoming, Body, Method, Uri};
use serde_json::json;
use std::{net::TcpListener, sync::Arc};
use tower_http::trace::TraceLayer;
use tracing::{info, warn};
use uuid::Uuid;

use crate::{configuration::Settings, email_client::EmailClient, error, routes};

type ZServer = Server<AddrIncoming, IntoMakeService<Router<(), Body>>>;

pub struct Application {
    port: u16,
    server: ZServer,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, std::io::Error> {
        let sender_email = configuration
            .email_client
            .sender()
            .expect("Invalid sender email address.");

        let timeout = configuration.email_client.timeout();
        let email_client = EmailClient::new(
            configuration.email_client.base_url.clone(),
            sender_email,
            configuration.email_client.authorization_token.clone(),
            timeout,
        );

        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = TcpListener::bind(&address)?;
        let port = listener.local_addr().unwrap().port();
        let server = run(listener, configuration, email_client).await?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), hyper::Error> {
        let quit_sig = async {
            _ = tokio::signal::ctrl_c().await;
            warn!("Received Ctrl-C, shutting down gracefully...");
        };

        self.server.with_graceful_shutdown(quit_sig).await
    }
}

#[derive(Clone)]
pub struct AppState {
    pub configuration: Settings,
    pub email_client: Arc<EmailClient>,
}

pub async fn run(
    listener: TcpListener,
    configuration: Settings,
    email_client: EmailClient,
) -> Result<ZServer, std::io::Error> {
    let state = AppState {
        configuration,
        email_client: Arc::new(email_client),
    };

    let app = Router::new()
        .route("/", get(routes::handler_hello))
        .route("/health_check", get(routes::handler_health_check))
        .route("/subscribe", post(routes::handler_subscribe))
        // .layer(middleware::map_response(main_response_mapper))
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &hyper::Request<Body>| {
                let uuid = Uuid::new_v4();
                tracing::info_span!(
                    "request",
                    uuid = %uuid,
                    method = %request.method(),
                    uri = %request.uri(),
                )
            }),
        )
        .layer(Extension(state));

    let server = Server::from_tcp(listener)
        .unwrap_or_else(|e| {
            panic!("Failed to bind random port: {}", e);
        })
        .serve(app.into_make_service());
    Ok(server)
}

#[allow(dead_code)]
async fn main_response_mapper(
    // db: Option<Surreal<Client>>,
    _uri: Uri,
    _req_method: Method,
    res: Response,
) -> Response {
    info!("->> {:<8} - main_response_mapper", "RES_MAPPER");
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

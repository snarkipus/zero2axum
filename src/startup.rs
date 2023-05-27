use axum::extract::FromRef;
#[allow(unused_imports)]
use axum::{
    extract::Extension,
    middleware,
    response::{IntoResponse, Response},
    routing::{get, post, IntoMakeService},
    Json, Router, Server,
};

use hyper::{server::conn::AddrIncoming, Body};
use std::{net::TcpListener, sync::Arc};
use tower_http::trace::TraceLayer;
use tracing::warn;
use uuid::Uuid;

use crate::{configuration::Settings, email_client::EmailClient, routes, routes::handler_confirm};

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
    pub base_url: ApplicationBaseUrl,
}

impl FromRef<AppState> for Settings {
    fn from_ref(state: &AppState) -> Settings {
        state.configuration.clone()
    }
}

impl FromRef<AppState> for Arc<EmailClient> {
    fn from_ref(state: &AppState) -> Arc<EmailClient> {
        state.email_client.clone()
    }
}

impl FromRef<AppState> for ApplicationBaseUrl {
    fn from_ref(state: &AppState) -> ApplicationBaseUrl {
        state.base_url.clone()
    }
}

#[derive(Clone)]
pub struct ApplicationBaseUrl(pub String);

pub async fn run(
    listener: TcpListener,
    configuration: Settings,
    email_client: EmailClient,
) -> Result<ZServer, std::io::Error> {
    let state = AppState {
        base_url: ApplicationBaseUrl(configuration.application.base_url.clone()),
        configuration,
        email_client: Arc::new(email_client),
    };

    let app = Router::new()
        .route("/", get(routes::handler_hello))
        .route("/health_check", get(routes::handler_health_check))
        .route("/subscribe", post(routes::handler_subscribe))
        .route("/subscribe/confirm", get(handler_confirm))
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
        .with_state(state);

    let server = Server::from_tcp(listener)
        .unwrap_or_else(|e| {
            panic!("Failed to bind random port: {}", e);
        })
        .serve(app.into_make_service());
    Ok(server)
}

use axum::extract::FromRef;
use axum::{
    routing::{get, post, IntoMakeService},
    Router, Server,
};
use color_eyre::Result;
use color_eyre::eyre::Context;
use hyper::{server::conn::AddrIncoming, Body};
use std::{net::TcpListener, sync::Arc};
use tower_http::trace::TraceLayer;
use tracing::warn;
use uuid::Uuid;


use crate::{
    configuration::Settings, db::Database, email_client::EmailClient, routes,
    routes::handler_confirm,
};

type ZServer = Server<AddrIncoming, IntoMakeService<Router<(), Body>>>;

// region: -- Application
pub struct Application {
    port: u16,
    server: ZServer,
}

impl Application {
    // region: -- Application Builder
    #[tracing::instrument(
        name = "Building Application",
        skip(configuration, database),
        fields(
            host = %configuration.application.host,
            port = %configuration.application.port,
        )
    )]
    pub async fn build(configuration: Settings, database: Database) -> Result<Self> {
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
        let listener = TcpListener::bind(&address).context("Failed to bind to address")?;
        let port = listener.local_addr().unwrap().port();
        let server = run(listener, configuration, email_client, database)
            .await
            .context("Server failed to run")?;

        Ok(Self { port, server })
    }
    // endregion: -- Application Builder

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<()> {
        let quit_sig = async {
            _ = tokio::signal::ctrl_c().await;
            warn!("Received Ctrl-C, shutting down gracefully...");
        };

        self.server
            .with_graceful_shutdown(quit_sig)
            .await
            .map_err(|e| color_eyre::Report::msg(format!("Server failed to run: {e}")))
    }
}
// endregion: -- Application

// region: -- AppState
#[derive(Clone)]
pub struct AppState {
    pub configuration: Settings,
    pub email_client: Arc<EmailClient>,
    pub base_url: ApplicationBaseUrl,
    pub database: Database,
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

impl FromRef<AppState> for Database {
    fn from_ref(state: &AppState) -> Database {
        state.database.clone()
    }
}

// endregion: -- AppState

#[derive(Clone)]
pub struct ApplicationBaseUrl(pub String);

pub async fn run(
    listener: TcpListener,
    configuration: Settings,
    email_client: EmailClient,
    database: Database,
) -> Result<ZServer> {
    let state = AppState {
        base_url: ApplicationBaseUrl(configuration.application.base_url.clone()),
        configuration,
        email_client: Arc::new(email_client),
        database,
    };

    let app = Router::new()
        .route("/", get(routes::handler_hello))
        .route("/health_check", get(routes::handler_health_check))
        .route("/subscribe", post(routes::handler_subscribe))
        .route("/subscribe/confirm", get(handler_confirm))
        .route("/newsletters", post(routes::publish_newsletter))
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
            panic!("Failed to bind random port: {e}");
        })
        .serve(app.into_make_service());
    Ok(server)
}

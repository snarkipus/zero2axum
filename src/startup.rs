use axum::extract::FromRef;
use axum::http::{Method, Uri};
use axum::middleware;
use axum::response::{IntoResponse, Response};
use axum::{
    routing::{get, post, IntoMakeService},
    Router, Server,
};
use color_eyre::Result;

use color_eyre::eyre::Context;
use hyper::StatusCode;
use hyper::{server::conn::AddrIncoming, Body};
use serde_json::json;
use std::{net::TcpListener, sync::Arc};
use tower_http::trace::TraceLayer;
use tracing::warn;
use uuid::Uuid;

use crate::error::{ConfirmationError, SubscribeError};
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
        .layer(middleware::map_response(main_response_mapper))
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

#[tracing::instrument(
    name = "Main Response Mapper",
    skip(uri, req_method, res),
    fields(
        uri = %uri,
        method = %req_method,
    )
)]
async fn main_response_mapper(uri: Uri, req_method: Method, res: Response) -> Response {
    let uuid = Uuid::new_v4();

    // Subscribe Response Error
    let res_err = res.extensions().get::<SubscribeError>();
    if let Some(res_err) = res_err {
        tracing::error!("Service Error: {:?}", res_err);
        match res_err {
            SubscribeError::UnexpectedError(_) => {
                return client_response_builder(StatusCode::INTERNAL_SERVER_ERROR, uuid)
            }
            SubscribeError::ValidationError(_) => {
                return client_response_builder(StatusCode::BAD_REQUEST, uuid)
            }
        }
    }

    // Subscribe Confirmation Response Error
    let res_err = res.extensions().get::<ConfirmationError>();
    if let Some(res_err) = res_err {
        tracing::error!("Service Error: {:?}", res_err);
        match res_err {
            ConfirmationError::UnexpectedError(_) => {
                return client_response_builder(StatusCode::INTERNAL_SERVER_ERROR, uuid)
            }
            ConfirmationError::UnknownToken => {
                return client_response_builder(StatusCode::UNAUTHORIZED, uuid)
            }
        }
    }

    // region: Client Error Response Builder
    #[tracing::instrument(
        name = "Client Error Response Builder",
        skip(status_code, uuid),
        fields(
            status = %status_code,
            id = %uuid,
        )
    )]
    fn client_response_builder(status_code: StatusCode, uuid: Uuid) -> Response {
        let client_response_body = json!({
            "error": {
                "type:": status_code.as_str(),
                "request_id": uuid.to_string(),
            }
        });
        tracing::error!("Client Error: {:?}", client_response_body);
        let res = Response::builder()
            .status(status_code)
            .header("Content-Type", "application/json")
            .body(Body::from(client_response_body.to_string()))
            .unwrap();
        res.into_response()
    }
    // endregion: -- Client Error Response Builder

    res
}

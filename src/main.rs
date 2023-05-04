use secrecy::ExposeSecret;
use std::net::TcpListener;
use surrealdb::{engine::remote::ws::Ws, opt::auth::Root, Surreal};
use tracing::{info, warn};

use zero2axum::{
    configuration::get_configuration,
    startup::run,
    telemetry::{get_subscriber, init_honeycomb, init_sentry, init_subscriber},
};

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let subscriber = get_subscriber("zero2axum".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    init_sentry();

    init_honeycomb();

    // region: -- SurrealDB: Initialize
    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection_string = format!(
        "{}:{}",
        configuration.database.host, configuration.database.port
    );

    let db = Surreal::new::<Ws>(connection_string)
        .await
        .expect("Failed to connect to SurrealDB.");

    db.signin(Root {
        username: &configuration.database.username,
        password: configuration.database.password.expose_secret(),
    })
    .await
    .expect("Failed to signin.");

    db.use_ns("default")
        .use_db("newsletter")
        .await
        .expect("Failed to use database.");
    // endregion: --- SurrealDB: Initialize

    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address).expect("Failed to bind to port");
    let port = listener.local_addr().unwrap().port();

    let quit_sig = async {
        _ = tokio::signal::ctrl_c().await;
        warn!("Received Ctrl-C, shutting down gracefully...");
    };

    let s = run(listener, db).await.unwrap_or_else(|e| {
        panic!("Failed to start server: {}", e);
    });
    info!("Server listening on http://127.0.0.1:{port}");
    s.with_graceful_shutdown(quit_sig).await?;

    Ok(())
}

use std::net::TcpListener;
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

    let configuration = get_configuration().expect("Failed to read configuration.");

    let address = format!("127.0.0.1:{}", configuration.application.port);
    let listener = TcpListener::bind(address).expect("Failed to bind to port");
    let port = listener.local_addr().unwrap().port();

    let quit_sig = async {
        _ = tokio::signal::ctrl_c().await;
        warn!("Received Ctrl-C, shutting down gracefully...");
    };

    let s = run(listener, configuration).await.unwrap_or_else(|e| {
        panic!("Failed to start server: {}", e);
    });
    info!("Server listening on http://127.0.0.1:{port}");
    s.with_graceful_shutdown(quit_sig).await?;

    Ok(())
}

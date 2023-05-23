use std::net::TcpListener;
use tracing::{info, warn};

use zero2axum::{
    configuration::get_configuration,
    email_client::EmailClient,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let subscriber = get_subscriber("zero2axum".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // init_sentry();

    // init_honeycomb();

    let configuration = get_configuration().expect("Failed to read configuration.");

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
    let listener = TcpListener::bind(&address).expect("Failed to bind to port");

    let quit_sig = async {
        _ = tokio::signal::ctrl_c().await;
        warn!("Received Ctrl-C, shutting down gracefully...");
    };

    let s = run(listener, configuration, email_client)
        .await
        .unwrap_or_else(|e| {
            panic!("Failed to start server: {}", e);
        });
    info!("Server listening on http://{address}");
    s.with_graceful_shutdown(quit_sig).await?;

    Ok(())
}

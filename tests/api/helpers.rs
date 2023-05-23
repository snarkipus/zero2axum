use once_cell::sync::Lazy;
use std::net::TcpListener;
use tracing::info;
use uuid::Uuid;
use zero2axum::{
    configuration::{get_configuration, Settings},
    email_client::EmailClient,
    telemetry::{get_subscriber, init_subscriber},
};

// region: -- conditional tracing for tests
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});
// endregion: -- conditional tracing for tests

// region: -- spawn_app
pub struct TestApp {
    pub configuration: Settings,
}

#[allow(clippy::let_underscore_future)]
pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let mut configuration = get_configuration().expect("Failed to read configuration.");
    configuration.database.database_name = Uuid::new_v4().to_string();

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

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    configuration.application.port = listener.local_addr().unwrap().port();

    let s = zero2axum::startup::run(listener, configuration.clone(), email_client)
        .await
        .unwrap_or_else(|e| {
            panic!("Failed to start server: {}", e);
        });
    info!(
        "Server listening on http://{}:{}",
        configuration.application.host, configuration.application.port
    );
    let _ = tokio::spawn(s);
    TestApp { configuration }
}
// endregion: -- spawn_app

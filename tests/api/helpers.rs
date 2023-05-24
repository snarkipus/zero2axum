use once_cell::sync::Lazy;
use uuid::Uuid;
use zero2axum::{
    configuration::{get_configuration, Settings},
    startup::Application,
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

    let mut configuration = {
        let mut c = get_configuration().expect("Failed to read configuration.");
        c.database.database_name = Uuid::new_v4().to_string();
        c.application.port = 0;
        c
    };

    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build application.");

    configuration.application.port = application.port();

    let _ = tokio::spawn(application.run_until_stopped());
    TestApp { configuration }
}
// endregion: -- spawn_app

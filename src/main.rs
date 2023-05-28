use tracing::info;
use zero2axum::{
    configuration::get_configuration,
    startup::Application,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let subscriber = get_subscriber("zero2axum".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);
    // init_sentry();
    // init_honeycomb();

    let configuration = get_configuration().expect("Failed to read configuration.");

    let application = Application::build(configuration.clone())
        .await
        .expect("Application Failed to Start");
    info!(
        "Server listening on http://{}:{}",
        configuration.application.host, configuration.application.port
    );
    application.run_until_stopped().await?;

    Ok(())
}

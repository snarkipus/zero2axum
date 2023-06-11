use color_eyre::eyre::Context;
use tracing::info;
use zero2axum::{
    configuration::get_configuration,
    db::Database,
    startup::Application,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let subscriber = get_subscriber("zero2axum".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);
    // init_sentry();
    // init_honeycomb();

    let configuration = get_configuration().expect("Failed to read configuration.");

    let database = Database::new(&configuration)
        .await
        .context("Application failed to create SurrealDB")?;

    let application = Application::build(configuration.clone(), database)
        .await
        .expect("Application Failed to Start");
    info!(
        "Server listening on http://{}:{}",
        configuration.application.host, configuration.application.port
    );
    application.run_until_stopped().await?;

    Ok(())
}

use std::{net::TcpListener, str::FromStr};
use surrealdb::{engine::remote::ws::Ws, opt::auth::Root, Surreal};
use tracing::{info, warn, Level};
use tracing_subscriber::{filter::Targets, layer::SubscriberExt, util::SubscriberInitExt};
use zero2axum::{configuration::get_configuration, startup::run};

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    // region: -- Tracing: Initialize
    let filter = Targets::from_str(std::env::var("RUST_LOG").as_deref().unwrap_or("info"))
        .expect("RUST_LOG must be a valid tracing filter");
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .pretty()
        // .json() // Uncomment to enable JSON logging
        .finish()
        .with(filter)
        .init();
    // endregion: --- Tracing: Initialize

    // region: -- Sentry.io error reporting
    let _guard = sentry::init((
        std::env::var("SENTRY_DSN").expect("$SENTRY_DSN must be set"),
        sentry::ClientOptions {
            release: sentry::release_name!(),
            ..Default::default()
        },
    ));
    // endregion: --- Sentry.io error reporting

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
        password: &configuration.database.password,
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

use std::{net::TcpListener, str::FromStr};

use tracing::{info, warn, Level};
use tracing_subscriber::{filter::Targets, layer::SubscriberExt, util::SubscriberInitExt};
use zero2axum::startup::run;

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

    let quit_sig = async {
        _ = tokio::signal::ctrl_c().await;
        warn!("Received Ctrl-C, shutting down gracefully...");
    };

    let listener = TcpListener::bind("127.0.0.1:3000").expect("Failed to bind to port");
    let port = listener.local_addr().unwrap().port();
    let s = run(listener).unwrap_or_else(|e| {
        panic!("Failed to start server: {}", e);
    });
    info!("Server listening on http://127.0.0.1:{port}");
    s.with_graceful_shutdown(quit_sig).await?;

    Ok(())
}

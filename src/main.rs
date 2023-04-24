use std::net::TcpListener;

use tracing::warn;

use zero2axum::run;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let quit_sig = async {
        _ = tokio::signal::ctrl_c().await;
        warn!("Received Ctrl-C, shutting down gracefully...");
    };

    let listener = TcpListener::bind("127.0.0.1:3000").expect("Failed to bind random port");
    // let port = listener.local_addr().unwrap().port();
    let s = run(listener).unwrap_or_else(|e| {
        panic!("Failed to start server: {}", e);
    });
    s.with_graceful_shutdown(quit_sig).await?;

    Ok(())
}

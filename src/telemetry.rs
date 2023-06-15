use tracing::subscriber::set_global_default;
use tracing::Subscriber;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{fmt::MakeWriter, layer::SubscriberExt, EnvFilter, Registry};
use tokio::task::JoinHandle;

// region: -- Spawn Blocking w/Tracing
pub fn spawn_block_with_tracing<F, R>(f: F) -> JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    let current_span = tracing::Span::current();
    tokio::task::spawn_blocking(move || current_span.in_scope(f))
}
// region: -- Spawn Blocking w/Tracing

// region: -- Tracing: Initialize
pub fn get_subscriber<Sink>(
    name: String,
    env_filter: String,
    sink: Sink,
) -> impl Subscriber + Send + Sync
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    let formatting_layer = BunyanFormattingLayer::new(name, sink);

    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    LogTracer::init().expect("Failed to set logger.");
    set_global_default(subscriber).expect("Failed to set subscriber.");
}
// endregion: --- Tracing: Initialize

// region: -- Sentry.io error reporting
pub fn init_sentry() {
    let _guard = sentry::init((
        std::env::var("SENTRY_DSN").expect("$SENTRY_DSN must be set"),
        sentry::ClientOptions {
            release: sentry::release_name!(),
            ..Default::default()
        },
    ));
}
// endregion: --- Sentry.io error reporting

// region: -- Honeycomb.io tracing
pub fn init_honeycomb() {
    let (_honeyguard, _tracer) = opentelemetry_honeycomb::new_pipeline(
        std::env::var("HONEYCOMB_API_KEY").expect("HONEYCOMB_API_KEY must be set"),
        "zero2axum".into(),
    )
    .install()
    .unwrap();
}
// endregion: --- Honeycomb.io tracing

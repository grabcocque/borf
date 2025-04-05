use std::path::Path;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    prelude::*,
    util::SubscriberInitExt,
    EnvFilter,
};
use tracing_tracy::TracyLayer;
use tracing_tree::HierarchicalLayer;

pub fn initialize_tracing(log_dir: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
    let log_dir = log_dir.as_ref();
    std::fs::create_dir_all(log_dir)?;

    let file_appender = tracing_appender::rolling::daily(log_dir, "parser.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let _guard = Box::leak(Box::new(_guard));

    let console_layer = HierarchicalLayer::default()
        .with_targets(true)
        .with_bracketed_fields(true);

    let file_layer = fmt::layer()
        .with_writer(non_blocking)
        .with_span_events(FmtSpan::ENTER | FmtSpan::EXIT)
        .with_target(true)
        .json();

    tracing_subscriber::registry()
        .with(console_layer.with_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::from("info")),
        ))
        .with(file_layer.with_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::from("trace")),
        ))
        .with(TracyLayer::default())
        .try_init()?;

    Ok(())
}

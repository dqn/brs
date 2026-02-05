use std::path::Path;

use anyhow::Result;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, fmt};

/// Initialize the logging system with tracing.
///
/// If `log_dir` is provided, logs will also be written to a file in that directory.
/// The `verbose` flag controls whether debug logs are shown.
pub fn init_logging(log_dir: Option<&Path>, verbose: bool) -> Result<()> {
    let filter = if verbose {
        EnvFilter::new("brs=debug,warn")
    } else {
        EnvFilter::new("brs=info,warn")
    };

    let registry = tracing_subscriber::registry().with(filter);

    if let Some(dir) = log_dir {
        let file_appender = RollingFileAppender::new(Rotation::DAILY, dir, "brs.log");
        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

        // Store guard in a static to prevent it from being dropped
        // This is safe because we only call init_logging once
        std::mem::forget(_guard);

        registry
            .with(fmt::layer().with_target(true))
            .with(fmt::layer().with_writer(non_blocking).with_ansi(false))
            .init();
    } else {
        registry.with(fmt::layer().with_target(true)).init();
    }

    Ok(())
}

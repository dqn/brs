use anyhow::Result;

/// Initialize logging with tracing-subscriber.
/// Respects RUST_LOG environment variable.
pub fn init_logging() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init()
        .map_err(|e| anyhow::anyhow!("failed to initialize logging: {}", e))?;
    Ok(())
}

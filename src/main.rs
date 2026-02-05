use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[command(name = "brs", version, about = "BMS rhythm game player")]
struct Cli {}

fn main() -> Result<()> {
    let _cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!("brs starting");

    Ok(())
}

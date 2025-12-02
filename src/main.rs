#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

use clap::Parser;

mod blob;

mod cli;
use cli::{Cli, Fork};

mod pacaya;
mod shasta;

mod wallet_provider;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    if let Ok(custom_env_file) = std::env::var("ENV_FILE") {
        // Try from custom env file, and abort if it fails
        dotenvy::from_filename(custom_env_file)?;
    } else {
        // Try from default .env file, and ignore if it fails. It might
        // be that the user isn't using it.
        dotenvy::dotenv().ok();
    }

    let cli = Cli::parse();

    match &cli.fork {
        Fork::Pacaya => pacaya::handle_command(cli).await,
        Fork::Shasta => shasta::handle_command(cli).await,
    }
}

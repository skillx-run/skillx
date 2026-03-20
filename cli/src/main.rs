use anyhow::Result;
use clap::Parser;

mod commands;

use commands::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Run(args) => commands::run::execute(args).await,
        Commands::Scan(args) => commands::scan::execute(args).await,
        Commands::Agents(args) => commands::agents::execute(args).await,
        Commands::Info(args) => commands::info::execute(args).await,
        Commands::Cache(args) => commands::cache::execute(args).await,
    };

    if let Err(e) = result {
        skillx::ui::error(&format!("{e:#}"));
        std::process::exit(1);
    }

    Ok(())
}

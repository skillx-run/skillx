use anyhow::Result;
use clap::Parser;

mod commands;

use commands::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Ensure base dirs exist
    let _ = skillx::config::Config::ensure_dirs();

    let result = match cli.command {
        Commands::Run(args) => commands::run::execute(args).await,
        Commands::Install(args) => commands::install::execute(args).await,
        Commands::Uninstall(args) => commands::uninstall::execute(args).await,
        Commands::List(args) => commands::list::execute(args).await,
        Commands::Update(args) => commands::update::execute(args).await,
        Commands::Init(args) => commands::init::execute(args).await,
        Commands::Scan(args) => commands::scan::execute(args).await,
        Commands::Agents(args) => commands::agents::execute(args).await,
        Commands::Info(args) => commands::info::execute(args).await,
        Commands::Cache(args) => commands::cache::execute(args).await,
    };

    if let Err(e) = result {
        // Check for user cancellation (exit cleanly)
        if e.downcast_ref::<skillx::error::SkillxError>()
            .is_some_and(|se| matches!(se, skillx::error::SkillxError::UserCancelled))
        {
            skillx::ui::info("Cancelled.");
            std::process::exit(0);
        }

        skillx::ui::error(&format!("{e:#}"));
        std::process::exit(1);
    }

    Ok(())
}

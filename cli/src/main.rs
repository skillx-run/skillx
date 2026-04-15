use anyhow::Result;
use clap::Parser;

mod commands;

use commands::{Cli, Commands};
use skillx::update_check;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Ensure base dirs exist
    let _ = skillx::config::Config::ensure_dirs();

    // Spawn background update check (non-blocking, skip for `upgrade` command)
    let is_upgrade = matches!(cli.command, Commands::Upgrade(_));
    let update_handle = if !is_upgrade {
        spawn_update_check()
    } else {
        None
    };

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
        Commands::Upgrade(args) => commands::upgrade::execute(args).await,
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
        print_update_notification(update_handle).await;
        std::process::exit(1);
    }

    print_update_notification(update_handle).await;

    Ok(())
}

/// Spawn a background update check if the rate-limit interval has elapsed.
/// Returns a JoinHandle if a network check was spawned, None otherwise.
fn spawn_update_check() -> Option<tokio::task::JoinHandle<Option<update_check::UpdateAvailable>>> {
    let config = skillx::config::Config::load().unwrap_or_default();
    if update_check::should_check(&config) {
        Some(tokio::spawn(async {
            update_check::check_for_update().await
        }))
    } else {
        None
    }
}

/// Await the background update check or fall back to cached result, then print notification.
/// Never panics or errors — all failures are silently swallowed.
async fn print_update_notification(
    handle: Option<tokio::task::JoinHandle<Option<update_check::UpdateAvailable>>>,
) {
    let update = if let Some(handle) = handle {
        // Background task was spawned — wait up to 3 seconds for result
        match tokio::time::timeout(std::time::Duration::from_secs(3), handle).await {
            Ok(Ok(result)) => result,
            _ => None, // timeout, join error, or no update
        }
    } else {
        // No background task — check cache for a known newer version
        update_check::cached_update_available()
    };

    if let Some(update) = update {
        eprintln!();
        eprintln!("{}", update_check::format_update_message(&update));
    }
}

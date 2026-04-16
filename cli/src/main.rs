use anyhow::Result;
use clap::Parser;

mod commands;

use commands::{Cli, Commands};
use skillx::config::Config;
use skillx::update_check;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Ensure base dirs exist
    let _ = Config::ensure_dirs();

    // Load config once — shared by both update check spawn and the cached fallback.
    let config = Config::load().unwrap_or_default();

    // Spawn background update check (non-blocking, skip for `upgrade` command)
    let is_upgrade = matches!(cli.command, Commands::Upgrade(_));
    let update_handle = if !is_upgrade {
        spawn_update_check(&config)
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
        // Suppress the "upgrade available" banner for the upgrade command itself —
        // it already reports its own upgrade status and a redundant banner is noisy.
        if !is_upgrade {
            print_update_notification(update_handle, &config).await;
        }
        std::process::exit(1);
    }

    if !is_upgrade {
        print_update_notification(update_handle, &config).await;
    }

    Ok(())
}

/// Spawn a background update check if the rate-limit interval has elapsed.
/// Returns a JoinHandle if a network check was spawned, None otherwise.
fn spawn_update_check(
    config: &Config,
) -> Option<tokio::task::JoinHandle<Option<update_check::UpdateAvailable>>> {
    if update_check::should_check(config) {
        Some(tokio::spawn(update_check::check_for_update()))
    } else {
        None
    }
}

/// Await the background update check or fall back to cached result, then print notification.
/// Never panics or errors — all failures are silently swallowed.
async fn print_update_notification(
    handle: Option<tokio::task::JoinHandle<Option<update_check::UpdateAvailable>>>,
    config: &Config,
) {
    let update = if let Some(mut handle) = handle {
        // Background task was spawned — wait up to 3 seconds for result, then abort.
        match tokio::time::timeout(std::time::Duration::from_secs(3), &mut handle).await {
            Ok(Ok(result)) => result,
            Ok(Err(_)) => None, // join error
            Err(_) => {
                // Cancel the still-running task so it doesn't outlive `main`.
                handle.abort();
                None
            }
        }
    } else {
        // No background task — check cache for a known newer version
        update_check::cached_update_available(config)
    };

    if let Some(update) = update {
        eprintln!();
        eprintln!("{}", update_check::format_update_message(&update));
    }
}

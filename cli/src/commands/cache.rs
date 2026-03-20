use clap::{Args, Subcommand};

use skillx::cache::CacheManager;
use skillx::ui;

#[derive(Args, Debug)]
pub struct CacheArgs {
    #[command(subcommand)]
    pub command: CacheCommands,
}

#[derive(Subcommand, Debug)]
pub enum CacheCommands {
    /// List cached skills
    Ls,
    /// Clean all cached skills
    Clean,
}

pub async fn execute(args: CacheArgs) -> anyhow::Result<()> {
    match args.command {
        CacheCommands::Ls => {
            let entries = CacheManager::list()?;
            if entries.is_empty() {
                ui::info("Cache is empty.");
                return Ok(());
            }

            ui::header("Cached Skills");
            for entry in &entries {
                let name = entry
                    .skill_name
                    .as_deref()
                    .unwrap_or("(unnamed)");
                eprintln!(
                    "  {} — {} (cached {})",
                    console::style(name).bold(),
                    entry.source,
                    entry.cached_at.format("%Y-%m-%d %H:%M"),
                );
            }
            eprintln!("\n  {} entries", entries.len());
        }
        CacheCommands::Clean => {
            let sp = ui::spinner("Cleaning cache...");
            let count = CacheManager::clean()?;
            sp.finish_and_clear();
            ui::success(&format!("Cleaned {count} cached entries."));
        }
    }

    Ok(())
}

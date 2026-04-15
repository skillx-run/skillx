pub mod agents;
pub mod cache;
pub mod info;
pub mod init;
pub mod install;
pub mod list;
pub mod run;
pub mod scan;
pub mod uninstall;
pub mod update;
pub mod upgrade;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "skillx",
    about = "npx for Agent Skills — fetch, scan, inject, run, clean in one command",
    version,
    propagate_version = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Fetch, scan, inject, and run a skill (temporary — auto-cleanup after use)
    Run(run::RunArgs),

    /// Install skill(s) persistently
    Install(install::InstallArgs),

    /// Uninstall skill(s)
    Uninstall(uninstall::UninstallArgs),

    /// List installed skills
    List(list::ListArgs),

    /// Update installed skill(s)
    Update(update::UpdateArgs),

    /// Initialize a new skillx.toml
    Init(init::InitArgs),

    /// Security scan a skill
    Scan(scan::ScanArgs),

    /// List detected agent environments
    Agents(agents::AgentsArgs),

    /// Show skill metadata
    Info(info::InfoArgs),

    /// Manage local cache
    Cache(cache::CacheArgs),

    /// Check for and upgrade skillx itself
    Upgrade(upgrade::UpgradeArgs),
}

pub mod agents;
pub mod cache;
pub mod info;
pub mod run;
pub mod scan;

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

    /// Security scan a skill
    Scan(scan::ScanArgs),

    /// List detected agent environments
    Agents(agents::AgentsArgs),

    /// Show skill metadata
    Info(info::InfoArgs),

    /// Manage local cache
    Cache(cache::CacheArgs),
}

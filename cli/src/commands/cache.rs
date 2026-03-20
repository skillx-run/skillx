use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub struct CacheArgs {
    #[command(subcommand)]
    pub command: CacheCommands,
}

#[derive(Subcommand, Debug)]
pub enum CacheCommands {
    /// List cached skills
    Ls,
    /// Clean cache
    Clean,
}

pub async fn execute(_args: CacheArgs) -> anyhow::Result<()> {
    skillx::ui::info("cache command not implemented yet");
    Ok(())
}

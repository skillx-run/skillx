use clap::Args;

#[derive(Args, Debug)]
pub struct AgentsArgs {
    /// Show all known agents (including undetected)
    #[arg(long)]
    pub all: bool,
}

pub async fn execute(_args: AgentsArgs) -> anyhow::Result<()> {
    skillx::ui::info("agents command not implemented yet");
    Ok(())
}

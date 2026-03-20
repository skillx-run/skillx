use clap::Args;

#[derive(Args, Debug)]
pub struct InfoArgs {
    /// Skill source to inspect
    pub source: String,
}

pub async fn execute(_args: InfoArgs) -> anyhow::Result<()> {
    skillx::ui::info("info command not implemented yet");
    Ok(())
}

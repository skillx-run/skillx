use clap::Args;

#[derive(Args, Debug)]
pub struct ScanArgs {
    /// Skill source to scan
    pub source: String,

    /// Output format
    #[arg(long, default_value = "text")]
    pub format: String,

    /// Fail threshold (info, warn, danger, block)
    #[arg(long, default_value = "danger")]
    pub fail_on: String,
}

pub async fn execute(_args: ScanArgs) -> anyhow::Result<()> {
    skillx::ui::info("scan command not implemented yet");
    Ok(())
}

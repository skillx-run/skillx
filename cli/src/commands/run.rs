use clap::Args;

#[derive(Args, Debug)]
pub struct RunArgs {
    /// Skill source (local path, github: prefix, or URL)
    pub source: String,

    /// Prompt to pass to the agent
    pub prompt: Option<String>,

    /// Read prompt from a file
    #[arg(short = 'f', long = "file")]
    pub prompt_file: Option<String>,

    /// Read prompt from stdin
    #[arg(long)]
    pub stdin: bool,

    /// Target agent (skip auto-detection)
    #[arg(long)]
    pub agent: Option<String>,

    /// Injection scope
    #[arg(long, default_value = "global")]
    pub scope: String,

    /// Attach files for the agent to use
    #[arg(long)]
    pub attach: Vec<String>,

    /// Force re-fetch (skip cache)
    #[arg(long)]
    pub no_cache: bool,

    /// Skip security scan (not recommended)
    #[arg(long)]
    pub skip_scan: bool,

    /// Auto-confirm WARN level risks
    #[arg(long)]
    pub yes: bool,

    /// Agent YOLO mode: pass permission-skip flags to the agent
    #[arg(long)]
    pub yolo: bool,

    /// Maximum run duration (e.g., "30m", "2h")
    #[arg(long)]
    pub timeout: Option<String>,
}

pub async fn execute(_args: RunArgs) -> anyhow::Result<()> {
    skillx::ui::info("run command not implemented yet");
    Ok(())
}

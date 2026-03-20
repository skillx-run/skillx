use clap::Args;

use skillx::scanner::{RiskLevel, ScanEngine};
use skillx::scanner::report::{JsonFormatter, SarifFormatter, TextFormatter};
use skillx::source::resolver;
use skillx::ui;

#[derive(Args, Debug)]
pub struct ScanArgs {
    /// Skill source to scan
    pub source: String,

    /// Output format (text, json, sarif)
    #[arg(long, default_value = "text")]
    pub format: String,

    /// Fail threshold (info, warn, danger, block)
    #[arg(long, default_value = "danger")]
    pub fail_on: String,
}

pub async fn execute(args: ScanArgs) -> anyhow::Result<()> {
    let fail_on: RiskLevel = args
        .fail_on
        .parse()
        .map_err(|e: String| anyhow::anyhow!(e))?;

    // Resolve source
    ui::step("Resolving source...");
    let fetched = resolver::resolve_and_fetch(&args.source, false).await?;

    // Run scan
    ui::step("Scanning...");
    let report = ScanEngine::scan(&fetched.dir)?;

    // Format output
    match args.format.as_str() {
        "json" => {
            println!("{}", JsonFormatter::format(&report));
        }
        "sarif" => {
            println!("{}", SarifFormatter::format(&report));
        }
        _ => {
            eprint!("{}", TextFormatter::format(&report));
        }
    }

    // Check fail threshold
    let overall = report.overall_level();
    if overall >= fail_on {
        return Err(anyhow::anyhow!(
            "scan failed: overall level {} meets or exceeds threshold {}",
            overall,
            fail_on
        ));
    }

    Ok(())
}

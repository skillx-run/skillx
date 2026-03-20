use clap::Args;

use skillx::scanner::{RiskLevel, ScanEngine};
use skillx::scanner::report::{JsonFormatter, TextFormatter};
use skillx::source;
use skillx::source::local::LocalSource;
use skillx::ui;

#[derive(Args, Debug)]
pub struct ScanArgs {
    /// Skill source to scan
    pub source: String,

    /// Output format (text, json)
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
    let skill_source = source::resolve(&args.source)?;

    let skill_dir = match &skill_source {
        source::SkillSource::Local(path) => {
            let resolved = LocalSource::fetch(path)?;
            resolved.root_dir
        }
        source::SkillSource::GitHub {
            owner,
            repo,
            path,
            ref_,
        } => {
            let sp = ui::spinner("Fetching from GitHub...");
            let cache_key = args.source.clone();

            // Check cache first
            if let Some(cached) = skillx::cache::CacheManager::lookup(&cache_key)? {
                sp.finish_and_clear();
                ui::success("Using cached copy");
                cached
            } else {
                skillx::config::Config::ensure_dirs()?;
                let dest = skillx::config::Config::cache_dir()
                    .join(skillx::cache::CacheManager::source_hash(&cache_key))
                    .join("skill-files");
                source::github::GitHubSource::fetch(
                    owner,
                    repo,
                    path.as_deref(),
                    ref_.as_deref(),
                    &dest,
                )
                .await?;
                sp.finish_and_clear();
                dest
            }
        }
    };

    // Run scan
    ui::step("Scanning...");
    let report = ScanEngine::scan(&skill_dir)?;

    // Format output
    match args.format.as_str() {
        "json" => {
            println!("{}", JsonFormatter::format(&report));
        }
        _ => {
            eprint!("{}", TextFormatter::format(&report));
        }
    }

    // Check fail threshold
    let overall = report.overall_level();
    if overall >= fail_on {
        std::process::exit(1);
    }

    Ok(())
}

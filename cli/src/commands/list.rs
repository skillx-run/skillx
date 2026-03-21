use clap::Args;

use skillx::config::Config;
use skillx::installed::InstalledState;
use skillx::source::resolver;
use skillx::ui;

#[derive(Args, Debug)]
pub struct ListArgs {
    /// Filter by agent
    #[arg(long)]
    pub agent: Option<String>,

    /// Filter by scope (project, global, or all)
    #[arg(long, default_value = "all")]
    pub scope: String,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,

    /// Check for outdated skills
    #[arg(long)]
    pub outdated: bool,
}

pub async fn execute(args: ListArgs) -> anyhow::Result<()> {
    let installed = InstalledState::load().unwrap_or_default();

    if installed.skills.is_empty() {
        if args.json {
            println!("[]");
        } else {
            ui::info("No skills installed.");
        }
        return Ok(());
    }

    // Filter
    let filtered: Vec<_> = installed
        .skills
        .iter()
        .filter(|s| {
            if let Some(ref agent_filter) = args.agent {
                s.injections.iter().any(|inj| inj.agent == *agent_filter)
            } else {
                true
            }
        })
        .filter(|s| {
            if args.scope == "all" {
                true
            } else {
                s.injections
                    .iter()
                    .any(|inj| inj.scope == args.scope)
            }
        })
        .collect();

    if args.json {
        let json = serde_json::to_string_pretty(&filtered)?;
        println!("{json}");
        return Ok(());
    }

    eprintln!("Installed skills ({}):\n", filtered.len());

    // Table header
    eprintln!(
        "{:<20} {:<45} {:<16} {}",
        "Name", "Source", "Agents", "Scope"
    );
    eprintln!(
        "{:<20} {:<45} {:<16} {}",
        "─".repeat(18),
        "─".repeat(43),
        "─".repeat(14),
        "─".repeat(7)
    );

    for skill in &filtered {
        let agents: Vec<&str> = skill
            .injections
            .iter()
            .map(|inj| inj.agent.as_str())
            .collect();
        let scopes: Vec<&str> = skill
            .injections
            .iter()
            .map(|inj| inj.scope.as_str())
            .collect();
        let agents_str = agents.join(", ");
        let scope_str = scopes
            .first()
            .copied()
            .unwrap_or("n/a");

        let source_display = if skill.source.len() > 43 {
            format!("{}...", &skill.source[..40])
        } else {
            skill.source.clone()
        };

        eprintln!(
            "{:<20} {:<45} {:<16} {}",
            skill.name, source_display, agents_str, scope_str
        );
    }

    // Outdated check
    if args.outdated {
        eprintln!();
        ui::step("Checking for updates...");
        let config = Config::load()?;

        for skill in &filtered {
            // Skip local sources
            if skill.source.starts_with('/')
                || skill.source.starts_with('.')
                || skill.source.starts_with('~')
            {
                continue;
            }

            match check_outdated(skill, &config).await {
                Ok(true) => {
                    ui::warn(&format!("{}: update available", skill.name));
                }
                Ok(false) => {
                    // Up to date, nothing to print
                }
                Err(e) => {
                    ui::warn(&format!("{}: check failed ({})", skill.name, e));
                }
            }
        }
    }

    Ok(())
}

/// Check if a skill has updates by comparing SHA256 hashes.
async fn check_outdated(
    skill: &skillx::installed::InstalledSkill,
    config: &Config,
) -> anyhow::Result<bool> {
    let fetched = resolver::resolve_and_fetch(&skill.source, true, config).await?;

    // Compute SHA256 set of newly fetched files
    let mut new_hashes: std::collections::HashSet<String> = std::collections::HashSet::new();
    collect_file_hashes(&fetched.dir, &mut new_hashes)?;

    // Collect installed file hashes
    let installed_hashes: std::collections::HashSet<String> = skill
        .injections
        .iter()
        .flat_map(|inj| inj.files.iter().map(|f| f.sha256.clone()))
        .collect();

    Ok(new_hashes != installed_hashes)
}

fn collect_file_hashes(
    dir: &std::path::Path,
    hashes: &mut std::collections::HashSet<String>,
) -> anyhow::Result<()> {
    use sha2::{Digest, Sha256};

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_file_hashes(&path, hashes)?;
        } else {
            let content = std::fs::read(&path)?;
            let mut hasher = Sha256::new();
            hasher.update(&content);
            hashes.insert(format!("{:x}", hasher.finalize()));
        }
    }
    Ok(())
}

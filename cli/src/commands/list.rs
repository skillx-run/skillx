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
        "{:<20} {:<12} {:<38} {:<16} Scope",
        "Name", "Version", "Source", "Agents"
    );
    eprintln!(
        "{:<20} {:<12} {:<38} {:<16} {}",
        "─".repeat(18),
        "─".repeat(10),
        "─".repeat(36),
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

        // Extract version from resolved_ref or source @ref
        let version = skill
            .resolved_ref
            .as_deref()
            .or_else(|| {
                skill.source.rsplit_once('@').map(|(_, v)| v)
            })
            .unwrap_or("-");

        let source_display = truncate_display(&skill.source, 36);

        eprintln!(
            "{:<20} {:<12} {:<38} {:<16} {}",
            skill.name, version, source_display, agents_str, scope_str
        );
    }

    // Outdated check
    if args.outdated {
        eprintln!();
        ui::step("Checking for updates...");
        let config = Config::load()?;

        let mut outdated_count = 0;
        for skill in &filtered {
            // Skip local sources
            if skillx::source::is_local_source(&skill.source) {
                continue;
            }

            match check_outdated(skill, &config).await {
                Ok(Some(info)) => {
                    ui::warn(&format!(
                        "{}: update available ({} files changed)",
                        skill.name, info.files_changed
                    ));
                    outdated_count += 1;
                }
                Ok(None) => {}
                Err(e) => {
                    ui::warn(&format!("{}: check failed ({})", skill.name, e));
                }
            }
        }
        if outdated_count > 0 {
            eprintln!();
            ui::info(
                "Run `skillx update` to update all, or `skillx update <name>` to update one."
            );
        } else {
            ui::success("All skills are up to date.");
        }
    }

    Ok(())
}

struct OutdatedInfo {
    files_changed: usize,
}

/// Check if a skill has updates by comparing (relative_path, sha256) pairs.
async fn check_outdated(
    skill: &skillx::installed::InstalledSkill,
    config: &Config,
) -> anyhow::Result<Option<OutdatedInfo>> {
    let fetched = resolver::resolve_and_fetch(&skill.source, true, config).await?;

    let new_hashes = skillx::installed::collect_file_hashes(&fetched.dir)?;

    // Collect installed (path, hash) pairs
    let installed_hashes: std::collections::BTreeSet<(String, String)> = skill
        .injections
        .iter()
        .flat_map(|inj| {
            inj.files
                .iter()
                .map(|f| (f.relative.clone(), f.sha256.clone()))
        })
        .collect();

    if new_hashes != installed_hashes {
        let files_changed: usize = new_hashes
            .symmetric_difference(&installed_hashes)
            .map(|(path, _)| path.as_str())
            .collect::<std::collections::BTreeSet<&str>>()
            .len();
        Ok(Some(OutdatedInfo { files_changed }))
    } else {
        Ok(None)
    }
}

/// Truncate a string for display, safe for multi-byte UTF-8.
fn truncate_display(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        return s.to_string();
    }
    // Find a safe char boundary to truncate at
    let truncate_at = max_len.saturating_sub(3);
    let end = s
        .char_indices()
        .take_while(|(i, _)| *i <= truncate_at)
        .last()
        .map(|(i, c)| i + c.len_utf8())
        .unwrap_or(0);
    format!("{}...", &s[..end])
}

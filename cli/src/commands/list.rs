use clap::Args;
use console::style;

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

        let mut outdated_entries: Vec<OutdatedInfo> = Vec::new();
        let mut checked_ok = 0usize;

        for skill in &filtered {
            // Skip local sources
            if skillx::source::is_local_source(&skill.source) {
                continue;
            }

            match check_outdated(skill, &config).await {
                Ok(Some(info)) => {
                    checked_ok += 1;
                    outdated_entries.push(info);
                }
                Ok(None) => {
                    checked_ok += 1;
                }
                Err(e) => {
                    ui::warn(&format!("{}: check failed ({})", skill.name, e));
                }
            }
        }

        if outdated_entries.is_empty() {
            ui::success("All skills are up to date.");
        } else {
            eprintln!();
            eprintln!(
                "{}",
                style(format!(
                    "Outdated skills ({} of {checked_ok} checked):",
                    outdated_entries.len()
                ))
                .bold()
            );
            eprintln!();
            eprintln!(
                "{:<20} {:<12} {:<12} {:<8} Source",
                "Name", "Installed", "Available", "Changed"
            );
            eprintln!(
                "{:<20} {:<12} {:<12} {:<8} {}",
                "─".repeat(18),
                "─".repeat(10),
                "─".repeat(10),
                "─".repeat(7),
                "─".repeat(30)
            );

            for entry in &outdated_entries {
                eprintln!(
                    "{:<20} {:<12} {:<12} {:<8} {}",
                    entry.name,
                    entry.installed_ref,
                    entry.available_ref,
                    format!("{} files", entry.files_changed),
                    truncate_display(&entry.source, 30)
                );
            }

            eprintln!();
            ui::info("Run `skillx update` to update all.");
        }
    }

    Ok(())
}

struct OutdatedInfo {
    name: String,
    installed_ref: String,
    available_ref: String,
    source: String,
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

        let installed_ref = skill
            .resolved_ref
            .as_deref()
            .or_else(|| skill.source.rsplit_once('@').map(|(_, v)| v))
            .unwrap_or("-")
            .to_string();

        let available_ref = fetched
            .resolved_ref
            .as_deref()
            .unwrap_or("-")
            .to_string();

        Ok(Some(OutdatedInfo {
            name: skill.name.clone(),
            installed_ref,
            available_ref,
            source: skill.source.clone(),
            files_changed,
        }))
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

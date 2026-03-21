use clap::Args;
use std::path::Path;

use skillx::cache::CacheManager;
use skillx::installed::InstalledState;
use skillx::project_config::ProjectConfig;
use skillx::ui;

use super::install::remove_injected_files;

#[derive(Args, Debug)]
pub struct UninstallArgs {
    /// Skill name(s) to uninstall
    #[arg(required = true)]
    pub names: Vec<String>,

    /// Only remove from a specific agent
    #[arg(long)]
    pub agent: Option<String>,

    /// Keep the entry in skillx.toml
    #[arg(long)]
    pub keep_in_toml: bool,

    /// Also purge cached files
    #[arg(long)]
    pub purge: bool,
}

pub async fn execute(args: UninstallArgs) -> anyhow::Result<()> {
    let mut installed = InstalledState::load().unwrap_or_default();

    for name in &args.names {
        let skill = installed
            .find_skill(name)
            .ok_or_else(|| anyhow::anyhow!("skill '{}' is not installed", name))?;

        if let Some(ref agent_filter) = args.agent {
            // Partial uninstall: only remove from specified agent
            let injection = skill
                .injections
                .iter()
                .find(|inj| inj.agent == *agent_filter);

            if let Some(inj) = injection {
                // Remove files for this agent's injection
                let base = Path::new(&inj.path);
                for file in &inj.files {
                    let path = base.join(&file.relative);
                    if path.exists() {
                        if let Err(e) = std::fs::remove_file(&path) {
                            ui::warn(&format!("Failed to remove {}: {e}", path.display()));
                        }
                    }
                }
                cleanup_empty_parents_for_injection(inj);

                let remaining_agents: Vec<String> = skill
                    .injections
                    .iter()
                    .filter(|i| i.agent != *agent_filter)
                    .map(|i| i.agent.clone())
                    .collect();

                installed.remove_injection(name, agent_filter);

                if remaining_agents.is_empty() {
                    ui::success(&format!("Uninstalled {name}"));
                } else {
                    ui::success(&format!(
                        "Removed {name} from {agent_filter}. Still installed in: {}",
                        remaining_agents.join(", ")
                    ));
                }
            } else {
                ui::warn(&format!("{name} is not installed in {agent_filter}"));
            }
        } else {
            // Full uninstall
            let source = skill.source.clone();
            let total_files: usize = skill.injections.iter().map(|inj| inj.files.len()).sum();
            let agents_list: Vec<&str> = skill
                .injections
                .iter()
                .map(|inj| inj.agent.as_str())
                .collect();
            let agents_display = agents_list.join(", ");
            remove_injected_files(skill);

            installed.remove_skill(name);
            ui::success(&format!(
                "Uninstalled {name} ({total_files} files removed from {agents_display})"
            ));

            // Purge cache
            if args.purge {
                let hash = CacheManager::source_hash(&source);
                let cache_dir = skillx::config::Config::cache_dir()?.join(&hash);
                if cache_dir.exists() {
                    match std::fs::remove_dir_all(&cache_dir) {
                        Ok(()) => ui::info(&format!("Purged cache for {name}")),
                        Err(e) => ui::warn(&format!("Failed to purge cache for {name}: {e}")),
                    }
                }
            }
        }

        // Update skillx.toml only if skill is fully removed (no remaining injections)
        if !args.keep_in_toml && !installed.is_installed(name) {
            if let Some(mut pc) = ProjectConfig::load(Path::new("."))? {
                if pc.remove_skill(name) {
                    pc.save(Path::new("."))?;
                    ui::info(&format!("Removed {name} from skillx.toml"));
                }
            }
        }
    }

    installed.save()?;
    Ok(())
}

fn cleanup_empty_parents_for_injection(injection: &skillx::installed::Injection) {
    let base = Path::new(&injection.path);
    let mut dirs: std::collections::BTreeSet<std::path::PathBuf> =
        std::collections::BTreeSet::new();

    for file in &injection.files {
        let full = base.join(&file.relative);
        let mut current = full.as_path();
        while let Some(parent) = current.parent() {
            if parent == base || parent.as_os_str().is_empty() {
                break;
            }
            dirs.insert(parent.to_path_buf());
            current = parent;
        }
    }
    dirs.insert(base.to_path_buf());

    let mut dirs: Vec<_> = dirs.into_iter().collect();
    dirs.sort_by_key(|b| std::cmp::Reverse(b.components().count()));

    for dir in dirs {
        if dir.exists() && dir.is_dir() {
            if let Ok(mut entries) = std::fs::read_dir(&dir) {
                if entries.next().is_none() {
                    std::fs::remove_dir(&dir).ok();
                }
            }
        }
    }
}

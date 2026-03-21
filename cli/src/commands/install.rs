use clap::Args;
use std::path::Path;

use skillx::agent::registry::AgentRegistry;
use skillx::config::Config;
use skillx::gate::gate_scan_result;
use skillx::installed::{InjectedFileRecord, Injection, InstalledSkill, InstalledState};
use skillx::project_config::ProjectConfig;
use skillx::scanner::report::TextFormatter;
use skillx::scanner::ScanEngine;
use skillx::session::inject::inject_and_collect;
use skillx::source::resolver;
use skillx::types::Scope;
use skillx::ui;

#[derive(Args, Debug)]
pub struct InstallArgs {
    /// Skill source(s) to install (local path, github: prefix, or URL)
    pub sources: Vec<String>,

    /// Target agent
    #[arg(long)]
    pub agent: Option<String>,

    /// Install to all detected agents
    #[arg(long)]
    pub all: bool,

    /// Injection scope
    #[arg(long, default_value = "global")]
    pub scope: String,

    /// Force re-fetch (skip cache)
    #[arg(long)]
    pub no_cache: bool,

    /// Skip security scan
    #[arg(long)]
    pub skip_scan: bool,

    /// Auto-confirm WARN level risks
    #[arg(long)]
    pub yes: bool,

    /// Don't save to skillx.toml
    #[arg(long)]
    pub no_save: bool,

    /// Install as dev dependency
    #[arg(long)]
    pub dev: bool,

    /// Only install production dependencies (skip dev)
    #[arg(long)]
    pub prod: bool,

    /// Remove installed skills not in skillx.toml
    #[arg(long)]
    pub prune: bool,
}

pub async fn execute(args: InstallArgs) -> anyhow::Result<()> {
    let config = Config::load()?;
    let mut installed = InstalledState::load().unwrap_or_default();

    if args.sources.is_empty() {
        // Manifest install mode: install from skillx.toml
        return install_from_toml(&args, &config, &mut installed).await;
    }

    // Explicit install mode
    let scope: Scope = args
        .scope
        .parse()
        .map_err(|e: String| anyhow::anyhow!(e))?;

    // Resolve and scan each source
    struct Resolved {
        dir: std::path::PathBuf,
        name: String,
        source: String,
        scan_level: String,
    }
    let mut resolved: Vec<Resolved> = Vec::new();

    for source_str in &args.sources {
        ui::step(&format!("Resolving: {source_str}"));
        let fetched = resolver::resolve_and_fetch(source_str, args.no_cache, &config).await?;
        ui::success(&format!("Resolved: {}", fetched.name));

        let scan_level = if !args.skip_scan {
            ui::step("Scanning...");
            let report = ScanEngine::scan(&fetched.dir)?;
            eprint!("{}", TextFormatter::format(&report));
            gate_scan_result(&Some(report.clone()), &fetched.dir, args.yes)?;
            format!("{}", report.overall_level())
        } else {
            ui::warn("Security scan skipped");
            "skipped".to_string()
        };

        resolved.push(Resolved {
            dir: fetched.dir,
            name: fetched.name,
            source: source_str.clone(),
            scan_level,
        });
    }

    // Select agents
    let registry = AgentRegistry::new(&config);
    let project_config = ProjectConfig::load(Path::new("."))?;
    let target_agents = select_agents(&args, &config, &registry, &project_config).await?;

    // Install each skill to each agent
    for skill in &resolved {
        for agent_name in &target_agents {
            let adapter = registry.get(agent_name).ok_or_else(|| {
                anyhow::anyhow!("agent not found: {agent_name}")
            })?;
            let inject_path = adapter.inject_path(&skill.name, &scope);

            // Conflict detection
            check_conflicts(&skill.name, &installed)?;

            ui::step(&format!(
                "Installing {} to {} ({})",
                skill.name,
                adapter.display_name(),
                scope
            ));

            let records = inject_and_collect(&skill.dir, &inject_path)?;
            let files: Vec<InjectedFileRecord> = records
                .iter()
                .map(|(rel, sha)| InjectedFileRecord {
                    relative: rel.clone(),
                    sha256: sha.clone(),
                })
                .collect();

            let injection = Injection {
                agent: agent_name.clone(),
                scope: scope.to_string(),
                path: inject_path.to_string_lossy().to_string(),
                files,
            };

            let now = chrono::Utc::now();
            if let Some(existing) = installed.find_skill_mut(&skill.name) {
                // Update existing: replace injection for this agent
                existing.injections.retain(|inj| inj.agent != *agent_name);
                existing.injections.push(injection);
                existing.updated_at = now;
                existing.source = skill.source.clone();
            } else {
                installed.add_or_update_skill(InstalledSkill {
                    name: skill.name.clone(),
                    source: skill.source.clone(),
                    resolved_ref: None,
                    installed_at: now,
                    updated_at: now,
                    scan_level: skill.scan_level.clone(),
                    injections: vec![injection],
                });
            }

            ui::success(&format!(
                "Installed {} ({} files) to {}",
                skill.name,
                records.len(),
                adapter.display_name()
            ));
        }
    }

    installed.save()?;

    // Save to skillx.toml if applicable
    if !args.no_save {
        if let Some(mut pc) = ProjectConfig::load(Path::new("."))? {
            for skill in &resolved {
                pc.add_skill(&skill.name, &skill.source, args.dev);
            }
            pc.save(Path::new("."))?;
            ui::info("Updated skillx.toml");
        }
    }

    Ok(())
}

async fn install_from_toml(
    args: &InstallArgs,
    config: &Config,
    installed: &mut InstalledState,
) -> anyhow::Result<()> {
    let pc = ProjectConfig::load(Path::new("."))?
        .ok_or_else(|| anyhow::anyhow!("No skillx.toml found. Provide source arguments or run 'skillx init'."))?;

    let scope: Scope = args
        .scope
        .parse()
        .map_err(|e: String| anyhow::anyhow!(e))?;

    let all_skills = pc.all_skills();
    let skills_to_install: Vec<_> = all_skills
        .iter()
        .filter(|(_, _, is_dev)| !args.prod || !is_dev)
        .collect();

    if skills_to_install.is_empty() {
        ui::info("No skills to install.");
        return Ok(());
    }

    let registry = AgentRegistry::new(config);
    let target_agents = select_agents(args, config, &registry, &Some(pc.clone())).await?;

    let mut count = 0;
    for (name, value, _is_dev) in &skills_to_install {
        let source = value.source();
        let skill_scope: Scope = value
            .scope()
            .unwrap_or(&args.scope)
            .parse()
            .unwrap_or(scope);

        // Skip if already installed with same source
        if let Some(existing) = installed.find_skill(name) {
            if existing.source == source {
                ui::info(&format!("{name} already installed, skipping"));
                continue;
            }
            ui::info(&format!("{name} source changed, updating"));
        }

        ui::step(&format!("Resolving: {source}"));
        let fetched = resolver::resolve_and_fetch(source, args.no_cache, config).await?;

        let skip_scan = value.skip_scan().unwrap_or(args.skip_scan);
        let scan_level = if !skip_scan {
            let report = ScanEngine::scan(&fetched.dir)?;
            eprint!("{}", TextFormatter::format(&report));
            gate_scan_result(&Some(report.clone()), &fetched.dir, args.yes)?;
            format!("{}", report.overall_level())
        } else {
            "skipped".to_string()
        };

        for agent_name in &target_agents {
            let adapter = registry.get(agent_name).ok_or_else(|| {
                anyhow::anyhow!("agent not found: {agent_name}")
            })?;
            let inject_path = adapter.inject_path(&fetched.name, &skill_scope);

            let records = inject_and_collect(&fetched.dir, &inject_path)?;
            let files: Vec<InjectedFileRecord> = records
                .iter()
                .map(|(rel, sha)| InjectedFileRecord {
                    relative: rel.clone(),
                    sha256: sha.clone(),
                })
                .collect();

            let now = chrono::Utc::now();
            if let Some(existing) = installed.find_skill_mut(name) {
                existing.injections.retain(|inj| inj.agent != *agent_name);
                existing.injections.push(Injection {
                    agent: agent_name.clone(),
                    scope: skill_scope.to_string(),
                    path: inject_path.to_string_lossy().to_string(),
                    files,
                });
                existing.updated_at = now;
                existing.source = source.to_string();
                existing.scan_level = scan_level.clone();
            } else {
                installed.add_or_update_skill(InstalledSkill {
                    name: name.clone(),
                    source: source.to_string(),
                    resolved_ref: None,
                    installed_at: now,
                    updated_at: now,
                    scan_level: scan_level.clone(),
                    injections: vec![Injection {
                        agent: agent_name.clone(),
                        scope: skill_scope.to_string(),
                        path: inject_path.to_string_lossy().to_string(),
                        files,
                    }],
                });
            }
        }
        count += 1;
        ui::success(&format!("Installed: {name}"));
    }

    // Prune: remove skills not in toml
    if args.prune {
        let toml_names: std::collections::HashSet<String> =
            skills_to_install.iter().map(|(n, _, _)| n.clone()).collect();
        let to_remove: Vec<String> = installed
            .skills
            .iter()
            .filter(|s| !toml_names.contains(&s.name))
            .map(|s| s.name.clone())
            .collect();
        for name in &to_remove {
            if let Some(skill) = installed.remove_skill(name) {
                remove_injected_files(&skill);
                ui::info(&format!("Pruned: {name}"));
            }
        }
    }

    installed.save()?;
    ui::success(&format!("Installed {count} skill(s)"));
    Ok(())
}

/// Select target agents based on args/config/detection.
async fn select_agents(
    args: &InstallArgs,
    config: &Config,
    registry: &AgentRegistry,
    project_config: &Option<ProjectConfig>,
) -> anyhow::Result<Vec<String>> {
    if let Some(ref agent) = args.agent {
        // Verify it exists
        registry.get(agent).ok_or_else(|| {
            anyhow::anyhow!("unknown agent: '{agent}'")
        })?;
        return Ok(vec![agent.clone()]);
    }

    if args.all {
        let detected = registry.detect_all().await;
        let agents: Vec<String> = detected
            .into_iter()
            .filter(|d| d.detected)
            .map(|d| d.name)
            .collect();
        if agents.is_empty() {
            return Err(anyhow::anyhow!("no agents detected"));
        }
        return Ok(agents);
    }

    // skillx.toml [agent].targets
    if let Some(pc) = project_config {
        if !pc.agent.targets.is_empty() {
            return Ok(pc.agent.targets.clone());
        }
    }

    // config.toml preferred (no interaction for install)
    if let Some(ref preferred) = config.agent.defaults.preferred {
        registry.get(preferred).ok_or_else(|| {
            anyhow::anyhow!("preferred agent not found: '{preferred}'")
        })?;
        return Ok(vec![preferred.clone()]);
    }

    // Auto-detect
    let adapter = registry.select(None).await.map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(vec![adapter.name().to_string()])
}

/// Check for conflicts before installing.
fn check_conflicts(skill_name: &str, installed: &InstalledState) -> anyhow::Result<()> {
    // Check for active sessions with this skill
    if let Ok(active_dir) = Config::active_dir() {
        if active_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&active_dir) {
                for entry in entries.flatten() {
                    let manifest_path = entry.path().join("manifest.json");
                    if manifest_path.exists() {
                        if let Ok(content) = std::fs::read_to_string(&manifest_path) {
                            if let Ok(manifest) = serde_json::from_str::<serde_json::Value>(&content) {
                                if manifest.get("skill_name").and_then(|v| v.as_str()) == Some(skill_name) {
                                    return Err(anyhow::anyhow!(
                                        "skill '{}' has an active run session. Wait for it to finish or clean up first.",
                                        skill_name
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Already installed -> treated as upgrade (no error)
    if installed.is_installed(skill_name) {
        ui::info(&format!("{skill_name} is already installed, upgrading"));
    }

    Ok(())
}

/// Remove injected files for an installed skill.
pub fn remove_injected_files(skill: &InstalledSkill) {
    for injection in &skill.injections {
        for file in &injection.files {
            let path = std::path::Path::new(&injection.path).join(&file.relative);
            if path.exists() {
                if let Err(e) = std::fs::remove_file(&path) {
                    ui::warn(&format!("Failed to remove {}: {e}", path.display()));
                }
            }
        }
        // Clean up empty directories
        cleanup_empty_parents(&injection.path, &injection.files);
    }
}

/// Clean up empty parent directories after file removal.
fn cleanup_empty_parents(base_path: &str, files: &[InjectedFileRecord]) {
    let base = std::path::Path::new(base_path);
    let mut dirs: std::collections::BTreeSet<std::path::PathBuf> = std::collections::BTreeSet::new();

    for file in files {
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
    // Also try to clean base itself
    dirs.insert(base.to_path_buf());

    // Sort deepest first
    let mut dirs: Vec<_> = dirs.into_iter().collect();
    dirs.sort_by(|a, b| b.components().count().cmp(&a.components().count()));

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

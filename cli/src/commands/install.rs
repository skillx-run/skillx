use clap::Args;
use std::path::Path;

use skillx::agent::registry::AgentRegistry;
use skillx::config::Config;
use skillx::gate::{gate_scan_result, GateOptions};
use skillx::installed::{InjectedFileRecord, Injection, InstalledSkill, InstalledState};
use skillx::project_config::ProjectConfig;
use skillx::scanner::report::TextFormatter;
use skillx::scanner::ScanEngine;
use skillx::session::inject;
use skillx::source::resolver;
use skillx::types::Scope;
use skillx::ui;

#[derive(Args, Debug)]
pub struct InstallArgs {
    /// Skill source(s) to install (local path, github: prefix, or URL)
    pub sources: Vec<String>,

    /// Target agent
    #[arg(long, conflicts_with = "all")]
    pub agent: Option<String>,

    /// Install to all detected agents
    #[arg(long, conflicts_with = "agent")]
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
    let scope: Scope = args.scope.parse().map_err(|e: String| anyhow::anyhow!(e))?;

    // Resolve and scan each source
    struct Resolved {
        dir: std::path::PathBuf,
        name: String,
        source: String,
        scan_level: String,
        resolved_ref: Option<String>,
    }

    // Phase 1: Concurrent fetch
    ui::step(&format!("Fetching {} skill(s)...", args.sources.len()));
    let fetch_futures = args.sources.iter().map(|source_str| {
        let source = source_str.clone();
        let no_cache = args.no_cache;
        let cfg = config.clone();
        async move {
            let result = resolver::resolve_and_fetch(&source, no_cache, &cfg).await;
            (source, result)
        }
    });
    let fetch_results = futures::future::join_all(fetch_futures).await;

    // Collect results, report any failures
    let mut fetched_skills = Vec::new();
    let mut fetch_errors = Vec::new();
    for (source, result) in fetch_results {
        match result {
            Ok(fetched) => {
                ui::success(&format!("Fetched {} from {}", fetched.name, source));
                fetched_skills.push((source, fetched));
            }
            Err(e) => {
                ui::error(&format!("Failed to fetch {source}: {e}"));
                fetch_errors.push((source, e));
            }
        }
    }
    if fetched_skills.is_empty() {
        return Err(anyhow::anyhow!(
            "all {} source(s) failed to fetch",
            fetch_errors.len()
        ));
    }
    if !fetch_errors.is_empty() {
        ui::warn(&format!(
            "{} of {} source(s) failed to fetch",
            fetch_errors.len(),
            fetch_errors.len() + fetched_skills.len()
        ));
    }

    // Phase 2: Sequential scan and gate (interactive)
    let mut resolved: Vec<Resolved> = Vec::new();
    for (source_str, fetched) in fetched_skills {
        let scan_level = if !args.skip_scan {
            ui::step(&format!("Scanning {}...", fetched.name));
            let report = ScanEngine::scan(&fetched.dir)?;
            eprint!("{}", TextFormatter::format(&report));
            gate_scan_result(&Some(report.clone()), &fetched.dir, &GateOptions { auto_yes: args.yes, headless: false })?;
            let findings_count = report.findings.len();
            if findings_count == 0 {
                ui::success("Security scan passed");
            } else {
                ui::success(&format!(
                    "Security scan: {} ({} findings)",
                    report.overall_level(),
                    findings_count
                ));
            }
            format!("{}", report.overall_level())
        } else {
            ui::warn("Security scan skipped");
            "skipped".to_string()
        };

        resolved.push(Resolved {
            dir: fetched.dir,
            name: fetched.name,
            source: source_str,
            scan_level,
            resolved_ref: fetched.resolved_ref,
        });
    }

    // Select agents
    let registry = AgentRegistry::new(&config);
    let project_config = ProjectConfig::load(Path::new("."))?;
    let target_agents = select_agents(&args, &config, &registry, &project_config).await?;

    // Install each skill to each agent
    for skill in &resolved {
        for agent_name in &target_agents {
            let adapter = registry
                .get(agent_name)
                .ok_or_else(|| anyhow::anyhow!("agent not found: {agent_name}"))?;
            let inject_path = adapter.inject_path(&skill.name, &scope);

            // Conflict detection
            check_conflicts(&skill.name, &inject_path, &installed)?;

            ui::step(&format!(
                "Installing {} to {} ({})",
                skill.name,
                adapter.display_name(),
                scope
            ));

            let records = adapter.prepare_injection(&skill.name, &skill.dir, &inject_path)?;
            let files: Vec<InjectedFileRecord> = records
                .iter()
                .filter(|r| r.injection_type == inject::InjectionType::CopiedFile)
                .map(|r| InjectedFileRecord {
                    relative: r.path.clone(),
                    sha256: r.sha256.clone(),
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
                existing.resolved_ref = skill.resolved_ref.clone();
                existing.scan_level = skill.scan_level.clone();
            } else {
                installed.add_or_update_skill(InstalledSkill {
                    name: skill.name.clone(),
                    source: skill.source.clone(),
                    resolved_ref: skill.resolved_ref.clone(),
                    resolved_commit: None,
                    installed_at: now,
                    updated_at: now,
                    scan_level: skill.scan_level.clone(),
                    injections: vec![injection],
                });
            }

            ui::success(&format!(
                "Installed to {} ({}: {})",
                adapter.display_name(),
                scope,
                inject_path.display()
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
    let pc = ProjectConfig::load(Path::new("."))?.ok_or_else(|| {
        anyhow::anyhow!("No skillx.toml found. Provide source arguments or run 'skillx init'.")
    })?;

    let scope: Scope = args.scope.parse().map_err(|e: String| anyhow::anyhow!(e))?;

    let all_skills = pc.all_skills();
    let skills_to_install: Vec<_> = all_skills
        .iter()
        .filter(|(_, _, is_dev)| !args.prod || !is_dev)
        .collect();

    if skills_to_install.is_empty() {
        ui::info("No skills to install.");
        return Ok(());
    }

    // Pre-scan: classify skills into new/update/unchanged
    let mut new_count = 0;
    let mut update_count = 0;
    let mut unchanged_count = 0;

    for (name, value, _is_dev) in &skills_to_install {
        let source = value.source();
        if let Some(existing) = installed.find_skill(name) {
            if existing.source == source {
                unchanged_count += 1;
            } else {
                update_count += 1;
            }
        } else {
            new_count += 1;
        }
    }

    ui::header(&format!(
        "Found {} skills ({} new, {} to update, {} already installed)",
        skills_to_install.len(),
        new_count,
        update_count,
        unchanged_count
    ));

    if new_count == 0 && update_count == 0 && !args.prune {
        ui::success("All skills are already installed.");
        return Ok(());
    }

    let registry = AgentRegistry::new(config);
    let target_agents = select_agents(args, config, &registry, &Some(pc.clone())).await?;

    // Phase 1: Concurrent fetch for new/changed skills
    let skills_to_fetch: Vec<_> = skills_to_install
        .iter()
        .filter(|(name, value, _)| {
            let source = value.source();
            if let Some(existing) = installed.find_skill(name) {
                existing.source != source
            } else {
                true
            }
        })
        .collect();

    let fetch_futures = skills_to_fetch.iter().map(|(name, value, _)| {
        let source = value.source().to_string();
        let no_cache = args.no_cache;
        let cfg = config.clone();
        let skill_name = name.clone();
        async move {
            let result = resolver::resolve_and_fetch(&source, no_cache, &cfg).await;
            (skill_name, source, result)
        }
    });
    let fetch_results = futures::future::join_all(fetch_futures).await;

    let mut fetched_map: std::collections::HashMap<String, _> = std::collections::HashMap::new();
    let mut toml_fetch_errors = 0usize;
    for (name, source, result) in fetch_results {
        match result {
            Ok(fetched) => {
                ui::success(&format!("Fetched {} from {}", fetched.name, source));
                fetched_map.insert(name, (source, fetched));
            }
            Err(e) => {
                ui::error(&format!("Failed to fetch {name} ({source}): {e}"));
                toml_fetch_errors += 1;
            }
        }
    }
    if fetched_map.is_empty() && toml_fetch_errors > 0 {
        return Err(anyhow::anyhow!(
            "all {} source(s) failed to fetch",
            toml_fetch_errors
        ));
    }
    if toml_fetch_errors > 0 {
        ui::warn(&format!(
            "{} of {} source(s) failed to fetch",
            toml_fetch_errors,
            toml_fetch_errors + fetched_map.len()
        ));
    }

    // Phase 2: Sequential scan/gate/inject
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
                continue;
            }
            ui::info(&format!("{name} source changed, updating"));
        }

        let (_source_str, fetched) = match fetched_map.remove(name) {
            Some(f) => f,
            None => continue, // fetch failed, already reported
        };

        let skip_scan = value.skip_scan().unwrap_or(args.skip_scan);
        let scan_level = if !skip_scan {
            let report = ScanEngine::scan(&fetched.dir)?;
            eprint!("{}", TextFormatter::format(&report));
            gate_scan_result(&Some(report.clone()), &fetched.dir, &GateOptions { auto_yes: args.yes, headless: false })?;
            format!("{}", report.overall_level())
        } else {
            "skipped".to_string()
        };

        for agent_name in &target_agents {
            let adapter = registry
                .get(agent_name)
                .ok_or_else(|| anyhow::anyhow!("agent not found: {agent_name}"))?;
            let inject_path = adapter.inject_path(&fetched.name, &skill_scope);

            let records = adapter.prepare_injection(&fetched.name, &fetched.dir, &inject_path)?;
            let files: Vec<InjectedFileRecord> = records
                .iter()
                .filter(|r| r.injection_type == inject::InjectionType::CopiedFile)
                .map(|r| InjectedFileRecord {
                    relative: r.path.clone(),
                    sha256: r.sha256.clone(),
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
                existing.resolved_ref = fetched.resolved_ref.clone();
                existing.scan_level = scan_level.clone();
            } else {
                installed.add_or_update_skill(InstalledSkill {
                    name: name.clone(),
                    source: source.to_string(),
                    resolved_ref: fetched.resolved_ref.clone(),
                    resolved_commit: None,
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
        let agents_display = target_agents.join(", ");
        ui::success(&format!("{name} is now available in {agents_display}"));
    }

    // Prune: remove skills not in toml (skip those with active sessions)
    if args.prune {
        let toml_names: std::collections::HashSet<String> = skills_to_install
            .iter()
            .map(|(n, _, _)| n.clone())
            .collect();
        let to_remove: Vec<String> = installed
            .skills
            .iter()
            .filter(|s| !toml_names.contains(&s.name))
            .map(|s| s.name.clone())
            .collect();
        for name in &to_remove {
            // Skip pruning if there's an active run session
            if check_conflicts(name, std::path::Path::new(""), installed).is_err() {
                ui::warn(&format!("Skipping prune of {name}: active session"));
                continue;
            }
            if let Some(skill) = installed.remove_skill(name) {
                remove_injected_files(&skill);
                ui::info(&format!("Pruned: {name}"));
            }
        }
    }

    installed.save()?;
    ui::success(&format!(
        "{} skills installed/updated, {} unchanged",
        count, unchanged_count
    ));
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
        registry
            .get(agent)
            .ok_or_else(|| anyhow::anyhow!("unknown agent: '{agent}'"))?;
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
        registry
            .get(preferred)
            .ok_or_else(|| anyhow::anyhow!("preferred agent not found: '{preferred}'"))?;
        return Ok(vec![preferred.clone()]);
    }

    // Auto-detect
    let adapter = registry
        .select(None)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(vec![adapter.name().to_string()])
}

/// Check for conflicts before installing.
/// Returns Ok(()) if safe to proceed.
fn check_conflicts(
    skill_name: &str,
    inject_path: &std::path::Path,
    installed: &InstalledState,
) -> anyhow::Result<()> {
    // Case 2: active run session → error
    if let Ok(active_dir) = Config::active_dir() {
        if active_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&active_dir) {
                for entry in entries.flatten() {
                    let manifest_path = entry.path().join("manifest.json");
                    if manifest_path.exists() {
                        if let Ok(content) = std::fs::read_to_string(&manifest_path) {
                            if let Ok(manifest) =
                                serde_json::from_str::<serde_json::Value>(&content)
                            {
                                if manifest.get("skill_name").and_then(|v| v.as_str())
                                    == Some(skill_name)
                                {
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

    // Case 1: already installed → upgrade
    if installed.is_installed(skill_name) {
        ui::info(&format!("{skill_name} is already installed, upgrading"));
        return Ok(());
    }

    // Case 3: target path exists but NOT managed by skillx → prompt overwrite
    if inject_path.exists() {
        ui::warn(&format!(
            "Found existing {} (not managed by skillx)",
            inject_path.display()
        ));
        eprint!("Overwrite? [y/N] ");
        std::io::Write::flush(&mut std::io::stderr()).ok();
        let mut input = String::new();
        std::io::BufRead::read_line(&mut std::io::stdin().lock(), &mut input)?;
        let input = input.trim().to_lowercase();
        if input != "y" && input != "yes" {
            return Err(skillx::error::SkillxError::UserCancelled.into());
        }
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
    let mut dirs: std::collections::BTreeSet<std::path::PathBuf> =
        std::collections::BTreeSet::new();

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

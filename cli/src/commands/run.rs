use clap::Args;
use console::style;
use std::io::{self, BufRead, Read};
use std::path::{Path, PathBuf};

use skillx::agent::registry::AgentRegistry;
use skillx::agent::{LaunchConfig, LifecycleMode};
use skillx::config::{self, Config};
use skillx::project_config::ProjectConfig;
use skillx::scanner::report::TextFormatter;
use skillx::scanner::{RiskLevel, ScanEngine};
use skillx::session::cleanup::{cleanup_session, recover_orphaned_sessions};
use skillx::session::inject::inject_skill;
use skillx::session::manifest::Manifest;
use skillx::session::Session;
use skillx::source::resolver;
use skillx::types::Scope;
use skillx::ui;

#[derive(Args, Debug)]
pub struct RunArgs {
    /// Skill source (local path, github: prefix, or URL). Optional if skillx.toml exists.
    pub source: Option<String>,

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

pub async fn execute(args: RunArgs) -> anyhow::Result<()> {
    Config::ensure_dirs()?;

    // Recover orphaned sessions from previous runs
    let orphans = recover_orphaned_sessions()?;
    if !orphans.is_empty() {
        ui::info(&format!(
            "Recovered {} orphaned session(s)",
            orphans.len()
        ));
    }

    // ── Phase 0: Load config ──
    let config = Config::load()?;
    let project_config = ProjectConfig::load(Path::new("."))?;

    // ── Phase 1: Determine source(s) ──
    // Priority: CLI source > skillx.toml skills > error
    let multi_skill_mode = args.source.is_none()
        && project_config
            .as_ref()
            .map(|pc| pc.has_skills())
            .unwrap_or(false);

    if args.source.is_none() && !multi_skill_mode {
        return Err(anyhow::anyhow!(
            "no source specified and no skillx.toml found. \
             Provide a source argument or create a skillx.toml with [skills] entries."
        ));
    }

    // Collect (source_string, skip_scan, scope_override) tuples
    let skill_entries: Vec<(String, bool, Option<String>)> = if let Some(ref source) = args.source {
        vec![(source.clone(), args.skip_scan, None)]
    } else {
        let pc = project_config.as_ref().unwrap();
        pc.all_skills()
            .iter()
            .map(|(_name, value, _is_dev)| {
                let skip = value
                    .skip_scan()
                    .unwrap_or(args.skip_scan);
                let scope = value
                    .scope()
                    .map(|s| s.to_string())
                    .or_else(|| pc.agent.scope.clone());
                (value.source().to_string(), skip, scope)
            })
            .collect()
    };

    // Resolved skill: (dir, name, scan_report, scope_override)
    struct ResolvedEntry {
        dir: PathBuf,
        name: String,
        scan_report: Option<skillx::scanner::ScanReport>,
        scope_override: Option<String>,
    }

    // Resolve, scan, gate each skill
    let mut resolved_skills: Vec<ResolvedEntry> = Vec::new();

    for (source_str, skip_scan, scope_override) in &skill_entries {
        ui::step(&format!("Resolving source: {source_str}"));
        let fetched = resolver::resolve_and_fetch(source_str, args.no_cache, &config).await?;
        let skill_dir = fetched.dir;
        let skill_name = fetched.name;
        ui::success(&format!("Resolved: {skill_name}"));

        // Scan
        let scan_report = if !skip_scan {
            ui::step("Scanning for security issues...");
            let report = ScanEngine::scan(&skill_dir)?;
            eprint!("{}", TextFormatter::format(&report));
            Some(report)
        } else {
            ui::warn("Security scan skipped (--skip-scan)");
            None
        };

        // Gate — check every skill, not just the first
        gate_scan_report(&scan_report, &skill_dir, args.yes)?;

        resolved_skills.push(ResolvedEntry {
            dir: skill_dir,
            name: skill_name,
            scan_report,
            scope_override: scope_override.clone(),
        });
    }

    // Split into primary + extras
    let primary = resolved_skills.remove(0);
    let extra_skills = resolved_skills;

    let skill_dir = primary.dir;
    let skill_name = primary.name;
    let scan_report = primary.scan_report;
    let primary_scope_override = primary.scope_override;

    // ── Phase 4: Detect Agent ──
    ui::step("Detecting agents...");
    let registry = AgentRegistry::new(&config);
    // Agent priority: CLI --agent > skillx.toml defaults.agent > config.toml preferred > auto-detect
    let agent_name = args
        .agent
        .as_deref()
        .or_else(|| {
            project_config
                .as_ref()
                .and_then(|pc| pc.agent.preferred.as_deref())
        })
        .or(config.agent.defaults.preferred.as_deref());
    let adapter = registry.select(agent_name).await?;
    ui::success(&format!("Using agent: {}", adapter.display_name()));

    // ── Phase 5: Parse scope ──
    // Per-skill scope: scope_override (from skillx.toml entry) > CLI --scope
    let scope: Scope = primary_scope_override
        .as_deref()
        .unwrap_or(&args.scope)
        .parse()
        .map_err(|e: String| anyhow::anyhow!(e))?;

    // ── Phase 6: Create session and inject ──
    ui::step("Injecting skill...");
    let session = Session::new(&skill_name);
    session.create_dirs()?;

    let inject_path = adapter.inject_path(&skill_name, &scope);
    let source_display = args.source.as_deref().unwrap_or("skillx.toml");
    let mut manifest = Manifest::new(
        &session.id,
        &skill_name,
        source_display,
        adapter.name(),
        &format!("{:?}", adapter.lifecycle_mode()),
        &scope.to_string(),
    );
    manifest.scan_result = scan_report;

    inject_skill(&skill_dir, &inject_path, &mut manifest)?;

    // Handle attachments (supports both files and directories)
    for attach in &args.attach {
        let src = PathBuf::from(attach);
        if !src.exists() {
            return Err(anyhow::anyhow!(
                "attachment not found: '{}'. Check that the path exists.",
                attach
            ));
        }
        if src.is_dir() {
            // Recursively copy directory
            let dir_name = src
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or("attachment".into());
            let dest_dir = inject_path.join("attachments").join(&dir_name);
            copy_dir_recursive(&src, &dest_dir)?;
            manifest.add_attachment(attach.clone(), dest_dir.to_string_lossy().to_string());
        } else {
            let filename = src
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or("attachment".into());
            let dest = inject_path.join("attachments").join(&filename);
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(&src, &dest)?;
            manifest.add_attachment(attach.clone(), dest.to_string_lossy().to_string());
        }
    }

    // Multi-skill mode: inject remaining skills with per-skill scope
    for entry in &extra_skills {
        let extra_scope: Scope = entry
            .scope_override
            .as_deref()
            .unwrap_or(&args.scope)
            .parse()
            .map_err(|e: String| anyhow::anyhow!(e))?;
        let extra_inject_path = adapter.inject_path(&entry.name, &extra_scope);
        inject_skill(&entry.dir, &extra_inject_path, &mut manifest)?;
        ui::success(&format!("Injected extra skill: {}", entry.name));
    }

    // Save manifest
    manifest.save(&Manifest::manifest_path(&session.session_dir()?))?;
    let total_skills = 1 + extra_skills.len();
    if total_skills > 1 {
        ui::success(&format!(
            "Injected {} skills ({} files total) to agent",
            total_skills,
            manifest.injected_files.len()
        ));
    } else {
        ui::success(&format!(
            "Injected {} files to {}",
            manifest.injected_files.len(),
            inject_path.display()
        ));
    }

    // ── Phase 7: Resolve prompt ──
    let prompt = resolve_prompt(&args)?;

    // ── Phase 8: Launch ──
    ui::step("Launching agent...");

    if args.yolo {
        if adapter.supports_yolo() {
            ui::warn(&format!(
                "YOLO mode: passing {}",
                adapter.yolo_args().join(" ")
            ));
        } else {
            ui::warn(&format!(
                "{} does not support YOLO mode — ignoring --yolo",
                adapter.display_name()
            ));
        }
    }

    let launch_config = LaunchConfig {
        skill_name: skill_name.clone(),
        skill_dir: inject_path.clone(),
        prompt,
        yolo: args.yolo,
        extra_args: vec![],
    };

    let mut session_handle = adapter.launch(launch_config).await?;

    // ── Phase 9: Wait (with Ctrl+C and timeout support) ──
    let timeout_duration = args
        .timeout
        .as_ref()
        .and_then(|t| config::parse_duration_secs(t))
        .map(|secs| {
            if secs < 5 {
                ui::warn("Timeout too short, using minimum of 5 seconds");
                std::time::Duration::from_secs(5)
            } else {
                std::time::Duration::from_secs(secs)
            }
        });

    match session_handle.lifecycle_mode {
        LifecycleMode::ManagedProcess => {
            if let Some(ref mut child) = session_handle.child {
                let wait_result: anyhow::Result<()> = tokio::select! {
                    result = child.wait() => {
                        match result {
                            Ok(status) if status.success() => {
                                ui::success("Agent completed successfully.");
                            }
                            Ok(status) => {
                                ui::warn(&format!(
                                    "Agent exited with code: {}",
                                    status.code().unwrap_or(-1)
                                ));
                            }
                            Err(e) => {
                                ui::error(&format!("Agent process error: {e}"));
                            }
                        }
                        Ok(())
                    }
                    _ = async {
                        if let Some(d) = timeout_duration {
                            tokio::time::sleep(d).await
                        } else {
                            // No timeout — never resolves
                            std::future::pending::<()>().await
                        }
                    } => {
                        ui::warn("Timeout reached. Terminating agent...");
                        child.kill().await.ok();
                        Ok(())
                    }
                    _ = tokio::signal::ctrl_c() => {
                        ui::info("Interrupted. Cleaning up...");
                        child.kill().await.ok();
                        Ok(())
                    }
                };
                wait_result?;
            }
        }
        LifecycleMode::FileInjectAndWait => {
            let wait_for_enter = tokio::task::spawn_blocking(|| {
                let mut input = String::new();
                io::stdin().lock().read_line(&mut input)
            });

            tokio::select! {
                _ = wait_for_enter => {
                    ui::success("Session complete.");
                }
                _ = async {
                    if let Some(d) = timeout_duration {
                        tokio::time::sleep(d).await
                    } else {
                        std::future::pending::<()>().await
                    }
                } => {
                    ui::warn("Timeout reached.");
                }
                _ = tokio::signal::ctrl_c() => {
                    ui::info("Interrupted. Cleaning up...");
                }
            }
        }
    }

    // ── Phase 10: Cleanup ──
    ui::step("Cleaning up...");
    cleanup_session(&session.session_dir()?)?;
    adapter.on_cleanup()?;
    ui::success("Cleanup complete.");

    Ok(())
}

/// Resolve prompt from: CLI arg > -f file > --stdin > None.
fn resolve_prompt(args: &RunArgs) -> anyhow::Result<Option<String>> {
    if let Some(ref prompt) = args.prompt {
        return Ok(Some(prompt.clone()));
    }

    if let Some(ref file) = args.prompt_file {
        let content = std::fs::read_to_string(file)?;
        return Ok(Some(content));
    }

    if args.stdin {
        let mut content = String::new();
        io::stdin().lock().read_to_string(&mut content)?;
        if !content.is_empty() {
            return Ok(Some(content));
        }
    }

    Ok(None)
}

/// Gate a scan report: check risk level and prompt user if needed.
fn gate_scan_report(
    scan_report: &Option<skillx::scanner::ScanReport>,
    skill_dir: &Path,
    auto_yes: bool,
) -> anyhow::Result<()> {
    let report = match scan_report {
        Some(r) => r,
        None => return Ok(()),
    };

    let level = report.overall_level();
    match level {
        RiskLevel::Pass | RiskLevel::Info => {}
        RiskLevel::Warn => {
            if !auto_yes {
                eprint!(
                    "{} Continue? [Y/n] ",
                    style("⚠").yellow().bold()
                );
                let mut input = String::new();
                io::stdin().lock().read_line(&mut input)?;
                let input = input.trim().to_lowercase();
                if input == "n" || input == "no" {
                    return Err(skillx::error::SkillxError::UserCancelled.into());
                }
            }
        }
        RiskLevel::Danger => {
            eprintln!(
                "\n{}",
                style("DANGER level findings detected. Review carefully.").red().bold()
            );
            eprintln!(
                "Type '{}' to see finding details, or type '{}' to continue:",
                style("detail N").cyan(),
                style("yes").green().bold()
            );

            let mut sorted_findings = report.findings.clone();
            sorted_findings.sort_by(|a, b| b.level.cmp(&a.level));

            loop {
                eprint!("{} ", style(">").dim());
                let mut input = String::new();
                io::stdin().lock().read_line(&mut input)?;
                let input = input.trim();

                if input.eq_ignore_ascii_case("yes") {
                    break;
                } else if input.eq_ignore_ascii_case("no") || input.eq_ignore_ascii_case("n") {
                    return Err(skillx::error::SkillxError::UserCancelled.into());
                } else if input.starts_with("detail") || input.starts_with("d ") {
                    let num_str = input
                        .strip_prefix("detail")
                        .or_else(|| input.strip_prefix("d "))
                        .unwrap_or("")
                        .trim();
                    if let Ok(n) = num_str.parse::<usize>() {
                        if n > 0 && n <= sorted_findings.len() {
                            let finding = &sorted_findings[n - 1];
                            eprintln!("\n{}", style("─".repeat(60)).dim());
                            eprintln!(
                                "  Rule:    {} ({})",
                                finding.rule_id,
                                finding.level
                            );
                            eprintln!("  File:    {}", finding.file);
                            if let Some(line) = finding.line {
                                eprintln!("  Line:    {line}");
                            }
                            eprintln!("  Message: {}", finding.message);

                            if let Some(line) = finding.line {
                                let file_path = skill_dir.join(&finding.file);
                                if let Ok(content) = std::fs::read_to_string(&file_path) {
                                    let lines: Vec<&str> = content.lines().collect();
                                    let start = line.saturating_sub(3);
                                    let end = (line + 2).min(lines.len());
                                    eprintln!("\n  Source:");
                                    for (i, l) in lines[start..end].iter().enumerate() {
                                        let line_num = start + i + 1;
                                        let marker = if line_num == line { ">" } else { " " };
                                        eprintln!(
                                            "  {marker} {}: {}",
                                            style(line_num).dim(),
                                            l
                                        );
                                    }
                                }
                            }
                            eprintln!("{}", style("─".repeat(60)).dim());
                        } else {
                            eprintln!("  Invalid finding number. Valid range: 1-{}", sorted_findings.len());
                        }
                    } else {
                        eprintln!("  Usage: detail <number>");
                    }
                } else {
                    eprintln!(
                        "  Type 'yes' to continue, 'no' to abort, or 'detail N' to inspect"
                    );
                }
            }
        }
        RiskLevel::Block => {
            ui::error("BLOCK level findings detected. Execution refused.");
            return Err(skillx::error::SkillxError::ScanBlocked.into());
        }
    }
    Ok(())
}

/// Recursively copy a directory to a destination, skipping symlinks.
fn copy_dir_recursive(src: &Path, dest: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(dest)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        // Skip symlinks to prevent following links outside the source tree
        if file_type.is_symlink() {
            continue;
        }
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_recursive(&src_path, &dest_path)?;
        } else {
            std::fs::copy(&src_path, &dest_path)?;
        }
    }
    Ok(())
}

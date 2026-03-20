use clap::Args;
use console::style;
use std::io::{self, BufRead, Read};
use std::path::PathBuf;

use skillx::agent::registry::AgentRegistry;
use skillx::agent::{LaunchConfig, LifecycleMode};
use skillx::cache::CacheManager;
use skillx::config::{self, Config};
use skillx::scanner::report::TextFormatter;
use skillx::scanner::{RiskLevel, ScanEngine};
use skillx::session::cleanup::{cleanup_session, recover_orphaned_sessions};
use skillx::session::inject::inject_skill;
use skillx::session::manifest::Manifest;
use skillx::session::Session;
use skillx::source;
use skillx::source::local::LocalSource;
use skillx::types::Scope;
use skillx::ui;

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

    // ── Phase 1: Resolve ──
    ui::step("Resolving source...");
    let skill_source = source::resolve(&args.source)?;

    let (skill_dir, skill_name) = match &skill_source {
        source::SkillSource::Local(path) => {
            let resolved = LocalSource::fetch(path)?;
            let name = resolved
                .metadata
                .name
                .clone()
                .unwrap_or_else(|| {
                    path.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or("skill".into())
                });
            (resolved.root_dir, name)
        }
        source::SkillSource::GitHub {
            owner,
            repo,
            path,
            ref_,
        } => {
            let cache_key = args.source.clone();
            let dir = if !args.no_cache {
                if let Some(cached) = CacheManager::lookup(&cache_key)? {
                    ui::success("Using cached copy");
                    cached
                } else {
                    fetch_github(owner, repo, path.as_deref(), ref_.as_deref(), &cache_key)
                        .await?
                }
            } else {
                fetch_github(owner, repo, path.as_deref(), ref_.as_deref(), &cache_key)
                    .await?
            };

            let resolved = LocalSource::fetch(&dir)?;
            let name = resolved
                .metadata
                .name
                .clone()
                .unwrap_or_else(|| path.as_deref().unwrap_or(repo).to_string());
            (dir, name)
        }
    };

    ui::success(&format!("Resolved: {skill_name}"));

    // ── Phase 2: Scan ──
    let scan_report = if !args.skip_scan {
        ui::step("Scanning for security issues...");
        let report = ScanEngine::scan(&skill_dir)?;
        eprint!("{}", TextFormatter::format(&report));
        Some(report)
    } else {
        ui::warn("Security scan skipped (--skip-scan)");
        None
    };

    // ── Phase 3: Gate ──
    if let Some(ref report) = scan_report {
        let level = report.overall_level();
        match level {
            RiskLevel::Pass | RiskLevel::Info => {
                // Auto-pass
            }
            RiskLevel::Warn => {
                if !args.yes {
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
                // Show detail interaction
                eprintln!(
                    "\n{}",
                    style("DANGER level findings detected. Review carefully.").red().bold()
                );
                eprintln!(
                    "Type '{}' to see finding details, or type '{}' to continue:",
                    style("detail N").cyan(),
                    style("yes").green().bold()
                );

                // Pre-sort findings once (highest severity first)
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

                                // Show file content if available
                                if let Some(line) = finding.line {
                                    let file_path = skill_dir.join(&finding.file);
                                    if let Ok(content) = std::fs::read_to_string(&file_path) {
                                        let lines: Vec<&str> = content.lines().collect();
                                        // 2 lines before + current + 2 lines after (1-indexed to 0-indexed)
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
    }

    // ── Phase 4: Detect Agent ──
    ui::step("Detecting agents...");
    let registry = AgentRegistry::new();
    let adapter = registry.select(args.agent.as_deref()).await?;
    ui::success(&format!("Using agent: {}", adapter.display_name()));

    // ── Phase 5: Parse scope ──
    let scope: Scope = args
        .scope
        .parse()
        .map_err(|e: String| anyhow::anyhow!(e))?;

    // ── Phase 6: Create session and inject ──
    ui::step("Injecting skill...");
    let session = Session::new(&skill_name);
    session.create_dirs()?;

    let inject_path = adapter.inject_path(&skill_name, &scope);
    let mut manifest = Manifest::new(
        &session.id,
        &skill_name,
        &args.source,
        adapter.name(),
        &format!("{:?}", adapter.lifecycle_mode()),
        &scope.to_string(),
    );
    // NOTE: scan_report is moved here. Phase 3 (Gate) borrows it above via `ref`,
    // which is fine because the borrow ends before this point. If you reorganize
    // the phases, ensure Gate completes before this move.
    manifest.scan_result = scan_report;

    inject_skill(&skill_dir, &inject_path, &mut manifest)?;

    // Handle attachments
    for attach in &args.attach {
        let src = PathBuf::from(attach);
        if !src.exists() {
            ui::warn(&format!("Attachment not found: {attach}"));
            continue;
        }
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

    // Save manifest
    manifest.save(&Manifest::manifest_path(&session.session_dir()?))?;
    ui::success(&format!(
        "Injected {} files to {}",
        manifest.injected_files.len(),
        inject_path.display()
    ));

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

async fn fetch_github(
    owner: &str,
    repo: &str,
    path: Option<&str>,
    ref_: Option<&str>,
    cache_key: &str,
) -> anyhow::Result<PathBuf> {
    let sp = ui::spinner("Fetching from GitHub...");
    let dest = Config::cache_dir()?
        .join(CacheManager::source_hash(cache_key))
        .join("skill-files");
    source::github::GitHubSource::fetch(owner, repo, path, ref_, &dest).await?;
    sp.finish_and_clear();
    ui::success("Downloaded from GitHub");
    Ok(dest)
}

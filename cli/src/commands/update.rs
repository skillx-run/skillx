use clap::Args;
use std::collections::BTreeSet;
use std::io::BufRead;
use std::path::Path;

use skillx::config::Config;
use skillx::gate::{gate_scan_result, GateOptions};
use skillx::installed::{InjectedFileRecord, InstalledState};
use skillx::project_config::ProjectConfig;
use skillx::scanner::report::TextFormatter;
use skillx::scanner::ScanEngine;
use skillx::session::inject::inject_and_collect;
use skillx::source::resolver;
use skillx::ui;

#[derive(Args, Debug)]
pub struct UpdateArgs {
    /// Skill name(s) to update (omit for all)
    pub names: Vec<String>,

    /// Only update for a specific agent
    #[arg(long)]
    pub agent: Option<String>,

    /// Show what would be updated without applying
    #[arg(long)]
    pub dry_run: bool,

    /// Skip security scan
    #[arg(long)]
    pub skip_scan: bool,

    /// Auto-confirm
    #[arg(long)]
    pub yes: bool,
}

pub async fn execute(args: UpdateArgs) -> anyhow::Result<()> {
    let config = Config::load()?;
    let mut installed = InstalledState::load().unwrap_or_default();

    if installed.skills.is_empty() {
        ui::info("No skills installed.");
        return Ok(());
    }

    // Determine which skills to check
    let skills_to_check: Vec<(String, String)> = if args.names.is_empty() {
        installed
            .skills
            .iter()
            .map(|s| (s.name.clone(), s.source.clone()))
            .collect()
    } else {
        let mut entries = Vec::new();
        for name in &args.names {
            let skill = installed
                .find_skill(name)
                .ok_or_else(|| anyhow::anyhow!("skill '{}' is not installed", name))?;
            entries.push((skill.name.clone(), skill.source.clone()));
        }
        entries
    };

    // Check each skill for updates
    struct UpdateCandidate {
        name: String,
        source: String,
        dir: std::path::PathBuf,
        resolved_ref: Option<String>,
        old_version: Option<String>,
        files_changed: usize,
        scan_level: String,
    }

    let mut candidates: Vec<UpdateCandidate> = Vec::new();

    // Separate local from remote sources
    let remote_skills: Vec<_> = skills_to_check
        .iter()
        .filter(|(name, source)| {
            if skillx::source::is_local_source(source) {
                ui::info(&format!("{name}: local source, skipping"));
                false
            } else {
                true
            }
        })
        .collect();

    // Concurrent fetch for remote sources
    ui::step(&format!(
        "Checking {} skill(s) for updates...",
        remote_skills.len()
    ));
    let fetch_futures = remote_skills.iter().map(|(name, source)| {
        let name = name.clone();
        let source = source.clone();
        let cfg = config.clone();
        async move {
            let result = resolver::resolve_and_fetch(&source, true, &cfg).await;
            (name, source, result)
        }
    });
    let fetch_results = futures::future::join_all(fetch_futures).await;

    // Compare results with installed state
    for (name, source, result) in fetch_results {
        match result {
            Ok(fetched) => {
                // Compare (path, hash) pairs for precise change detection
                let new_hashes = match skillx::installed::collect_file_hashes(&fetched.dir) {
                    Ok(h) => h,
                    Err(e) => {
                        ui::warn(&format!("{name}: hash check failed ({e})"));
                        continue;
                    }
                };

                let skill = installed.find_skill(&name).ok_or_else(|| {
                    anyhow::anyhow!(
                        "internal: skill '{}' disappeared from installed state",
                        name
                    )
                })?;
                let installed_hashes: BTreeSet<(String, String)> = skill
                    .injections
                    .iter()
                    .flat_map(|inj| {
                        inj.files
                            .iter()
                            .map(|f| (f.relative.clone(), f.sha256.clone()))
                    })
                    .collect();

                if new_hashes != installed_hashes {
                    let old_version = skill
                        .resolved_ref
                        .as_deref()
                        .or_else(|| skill.source.rsplit_once('@').map(|(_, v)| v))
                        .map(|s| s.to_string());
                    let files_changed: usize = new_hashes
                        .symmetric_difference(&installed_hashes)
                        .map(|(path, _)| path.as_str())
                        .collect::<BTreeSet<&str>>()
                        .len();
                    ui::info(&format!("{name}: update available"));
                    candidates.push(UpdateCandidate {
                        name: name.clone(),
                        source: source.clone(),
                        dir: fetched.dir,
                        resolved_ref: fetched.resolved_ref,
                        old_version,
                        files_changed,
                        scan_level: String::new(), // filled during apply
                    });
                }
            }
            Err(e) => {
                ui::warn(&format!("{name}: check failed ({e})"));
            }
        }
    }

    if candidates.is_empty() {
        ui::success("All skills are up to date.");
        return Ok(());
    }

    // Show summary table
    eprintln!(
        "\n{:<20} {:<12} {:<12} Change",
        "Name", "Installed", "Available"
    );
    eprintln!(
        "{:<20} {:<12} {:<12} {}",
        "─".repeat(18),
        "─".repeat(10),
        "─".repeat(10),
        "─".repeat(20)
    );
    for c in &candidates {
        let old_ver = c.old_version.as_deref().unwrap_or("-");
        let new_ver = c.resolved_ref.as_deref().unwrap_or("-");
        eprintln!(
            "{:<20} {:<12} {:<12} {} files changed",
            c.name, old_ver, new_ver, c.files_changed
        );
    }

    if args.dry_run {
        ui::info("Dry run — no changes applied.");
        return Ok(());
    }

    // Confirm
    if !args.yes {
        eprint!("\nUpdate {} skill(s)? [Y/n] ", candidates.len());
        std::io::Write::flush(&mut std::io::stderr()).ok();
        let mut input = String::new();
        std::io::stdin().lock().read_line(&mut input)?;
        let input = input.trim().to_lowercase();
        if input == "n" || input == "no" {
            return Err(skillx::error::SkillxError::UserCancelled.into());
        }
    }

    // Apply updates (save after each success to avoid losing progress on failure)
    let mut updated_count = 0;
    for candidate in &mut candidates {
        ui::step(&format!("Updating {}...", candidate.name));

        // Scan
        candidate.scan_level = if !args.skip_scan {
            let report = ScanEngine::scan(&candidate.dir)?;
            eprint!("{}", TextFormatter::format(&report));
            if let Err(e) = gate_scan_result(
                &Some(report.clone()),
                &candidate.dir,
                &GateOptions {
                    auto_yes: args.yes,
                    headless: false,
                },
            ) {
                // Save progress before propagating scan gate error
                if updated_count > 0 {
                    installed.save().ok();
                }
                return Err(e);
            }
            format!("{}", report.overall_level())
        } else {
            "skipped".to_string()
        };

        let skill = installed.find_skill_mut(&candidate.name).ok_or_else(|| {
            anyhow::anyhow!(
                "internal: skill '{}' disappeared from installed state",
                candidate.name
            )
        })?;

        // Update each injection
        for injection in &mut skill.injections {
            if let Some(ref agent_filter) = args.agent {
                if injection.agent != *agent_filter {
                    continue;
                }
            }

            let inject_path = std::path::Path::new(&injection.path);

            // Remove old files
            for file in &injection.files {
                let path = inject_path.join(&file.relative);
                if path.exists() {
                    std::fs::remove_file(&path).ok();
                }
            }

            // Inject new files
            let records = inject_and_collect(&candidate.dir, inject_path)?;
            injection.files = records
                .iter()
                .map(|(rel, sha)| InjectedFileRecord {
                    relative: rel.clone(),
                    sha256: sha.clone(),
                })
                .collect();
        }

        skill.updated_at = chrono::Utc::now();
        skill.source = candidate.source.clone();
        skill.resolved_ref = candidate.resolved_ref.clone();
        skill.scan_level = candidate.scan_level.clone();
        updated_count += 1;

        ui::success(&format!("Updated {}", candidate.name));
    }

    installed.save()?;

    // Sync skillx.toml: update source strings for skills whose source changed
    if let Some(mut pc) = ProjectConfig::load(Path::new("."))? {
        let mut toml_updated = false;
        for candidate in &candidates {
            if pc.update_skill_source(&candidate.name, &candidate.source) {
                toml_updated = true;
            }
        }
        if toml_updated {
            pc.save(Path::new("."))?;
            ui::info("Updated skillx.toml");
        }
    }

    ui::success(&format!("Updated {} skill(s)", updated_count));
    Ok(())
}

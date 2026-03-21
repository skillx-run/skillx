use clap::Args;
use std::collections::BTreeSet;
use std::io::BufRead;

use skillx::config::Config;
use skillx::gate::gate_scan_result;
use skillx::installed::{InjectedFileRecord, InstalledState};
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
            let skill = installed.find_skill(name).ok_or_else(|| {
                anyhow::anyhow!("skill '{}' is not installed", name)
            })?;
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
    }

    let mut candidates: Vec<UpdateCandidate> = Vec::new();

    ui::step("Checking for updates...");
    for (name, source) in &skills_to_check {
        // Skip local sources
        if skillx::source::is_local_source(source) {
            ui::info(&format!("{name}: local source, skipping"));
            continue;
        }

        match resolver::resolve_and_fetch(source, true, &config).await {
            Ok(fetched) => {
                // Compare (path, hash) pairs for precise change detection
                let new_hashes = skillx::installed::collect_file_hashes(&fetched.dir)?;

                let skill = installed.find_skill(name).unwrap();
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
                    let old_version = skill.resolved_ref.as_deref()
                        .or_else(|| skill.source.rsplit_once('@').map(|(_, v)| v))
                        .map(|s| s.to_string());
                    let files_changed = new_hashes.symmetric_difference(&installed_hashes).count();
                    ui::info(&format!("{name}: update available"));
                    candidates.push(UpdateCandidate {
                        name: name.clone(),
                        source: source.clone(),
                        dir: fetched.dir,
                        resolved_ref: fetched.resolved_ref,
                        old_version,
                        files_changed,
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
    eprintln!("\n{:<20} {:<12} Source", "Name", "Status");
    eprintln!("{:<20} {:<12} {}", "─".repeat(18), "─".repeat(10), "─".repeat(30));
    for c in &candidates {
        eprintln!("{:<20} {:<12} {}", c.name, "outdated", c.source);
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
    for candidate in &candidates {
        ui::step(&format!("Updating {}...", candidate.name));

        // Scan
        if !args.skip_scan {
            let report = ScanEngine::scan(&candidate.dir)?;
            eprint!("{}", TextFormatter::format(&report));
            if let Err(e) = gate_scan_result(&Some(report.clone()), &candidate.dir, args.yes) {
                // Save progress before propagating scan gate error
                if updated_count > 0 {
                    installed.save().ok();
                }
                return Err(e);
            }
        }

        let skill = installed.find_skill_mut(&candidate.name).unwrap();

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
        updated_count += 1;

        ui::success(&format!("Updated {}", candidate.name));
    }

    installed.save()?;
    ui::success(&format!("Updated {} skill(s)", updated_count));
    Ok(())
}


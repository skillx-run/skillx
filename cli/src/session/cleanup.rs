use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::error::{Result, SkillxError};
use crate::ui;

use super::manifest::Manifest;

/// Clean up a session: remove injected files, archive manifest.
pub fn cleanup_session(session_dir: &Path) -> Result<()> {
    let manifest_path = Manifest::manifest_path(session_dir);
    if !manifest_path.exists() {
        ui::warn(&format!(
            "No manifest found in {}",
            session_dir.display()
        ));
        return Ok(());
    }

    let manifest = Manifest::load(&manifest_path)?;

    // Remove injected files (with mtime change detection)
    for file in &manifest.injected_files {
        let path = PathBuf::from(&file.path);
        if path.exists() {
            // Check if file was modified by user
            if let Ok(content) = std::fs::read(&path) {
                let mut hasher = Sha256::new();
                hasher.update(&content);
                let current_sha = format!("{:x}", hasher.finalize());
                if current_sha != file.sha256 {
                    ui::warn(&format!(
                        "File was modified during session: {}",
                        file.path
                    ));
                }
            }

            if let Err(e) = std::fs::remove_file(&path) {
                ui::warn(&format!("Failed to remove {}: {e}", file.path));
            }
        }
    }

    // Remove injected attachments
    for attachment in &manifest.injected_attachments {
        let path = PathBuf::from(&attachment.copied_to);
        if path.exists() {
            if let Err(e) = std::fs::remove_file(&path) {
                ui::warn(&format!("Failed to remove {}: {e}", attachment.copied_to));
            }
        }
    }

    // Clean up empty directories
    cleanup_empty_dirs_from_files(&manifest)?;

    // Archive session to history
    archive_session(session_dir, &manifest)?;

    // Remove session directory
    if session_dir.exists() {
        std::fs::remove_dir_all(session_dir).ok();
    }

    Ok(())
}

/// Remove empty directories that were created by file injection.
/// Collects all ancestor directories (not just direct parents) so the
/// entire injection tree can be cleaned up if empty.
fn cleanup_empty_dirs_from_files(manifest: &Manifest) -> Result<()> {
    let mut dir_set = HashSet::new();

    for file in &manifest.injected_files {
        let path = PathBuf::from(&file.path);
        // Walk up the parent chain to collect all ancestor dirs
        let mut current = path.as_path();
        while let Some(parent) = current.parent() {
            // Stop at filesystem root or current directory
            if parent == current || parent.as_os_str().is_empty() {
                break;
            }
            dir_set.insert(parent.to_path_buf());
            current = parent;
        }
    }

    // Sort by depth (deepest first) so we remove inner dirs before outer ones
    let mut dirs: Vec<PathBuf> = dir_set.into_iter().collect();
    dirs.sort_by(|a, b| {
        let a_depth = a.components().count();
        let b_depth = b.components().count();
        b_depth.cmp(&a_depth)
    });

    for dir in &dirs {
        if dir.exists() && dir.is_dir() {
            if let Ok(mut entries) = std::fs::read_dir(dir) {
                if entries.next().is_none() {
                    std::fs::remove_dir(dir).ok();
                }
            }
        }
    }

    Ok(())
}

/// Archive session manifest to history.
fn archive_session(_session_dir: &Path, manifest: &Manifest) -> Result<()> {
    let history_dir = Config::history_dir()?;
    std::fs::create_dir_all(&history_dir)
        .map_err(|e| SkillxError::Session(format!("failed to create history dir: {e}")))?;

    let archive_path = history_dir.join(format!("{}.json", manifest.session_id));
    manifest.save(&archive_path)?;

    // Trim old history entries
    trim_history(&history_dir, 50)?;

    Ok(())
}

/// Keep only the most recent N history entries.
fn trim_history(history_dir: &Path, max_entries: usize) -> Result<()> {
    let mut entries: Vec<_> = std::fs::read_dir(history_dir)
        .map_err(|e| SkillxError::Session(format!("failed to read history dir: {e}")))?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "json")
                .unwrap_or(false)
        })
        .collect();

    if entries.len() <= max_entries {
        return Ok(());
    }

    // Sort by modification time (oldest first)
    entries.sort_by(|a, b| {
        let a_time = a.metadata().and_then(|m| m.modified()).ok();
        let b_time = b.metadata().and_then(|m| m.modified()).ok();
        a_time.cmp(&b_time)
    });

    let to_remove = entries.len() - max_entries;
    for entry in entries.iter().take(to_remove) {
        if let Err(e) = std::fs::remove_file(entry.path()) {
            ui::warn(&format!(
                "Failed to remove history entry {}: {e}",
                entry.path().display()
            ));
        }
    }

    Ok(())
}

/// Recover orphaned sessions from `~/.skillx/active/`.
///
/// When `interactive` is true, shows session metadata and asks for confirmation.
/// When false (e.g., `--yes` mode), automatically cleans up.
pub fn recover_orphaned_sessions() -> Result<Vec<String>> {
    recover_orphaned_sessions_inner(false)
}

/// Interactive version of orphaned session recovery.
pub fn recover_orphaned_sessions_interactive() -> Result<Vec<String>> {
    recover_orphaned_sessions_inner(true)
}

fn recover_orphaned_sessions_inner(interactive: bool) -> Result<Vec<String>> {
    let active_dir = Config::active_dir()?;
    if !active_dir.exists() {
        return Ok(vec![]);
    }

    let mut orphan_entries = Vec::new();
    let entries = std::fs::read_dir(&active_dir)
        .map_err(|e| SkillxError::Session(format!("failed to read active dir: {e}")))?;

    for entry in entries {
        let entry = entry
            .map_err(|e| SkillxError::Session(format!("dir entry error: {e}")))?;
        if entry.path().is_dir() {
            orphan_entries.push(entry);
        }
    }

    if orphan_entries.is_empty() {
        return Ok(vec![]);
    }

    // Collect session metadata for display
    if interactive {
        eprintln!(
            "\n{} Found orphaned session(s) from previous runs:",
            console::style("\u{26a0}").yellow().bold()
        );
        for (i, entry) in orphan_entries.iter().enumerate() {
            let session_id = entry.file_name().to_string_lossy().to_string();
            let short_id = &session_id[..8.min(session_id.len())];
            let manifest_path = Manifest::manifest_path(&entry.path());
            let detail = if manifest_path.exists() {
                if let Ok(manifest) = Manifest::load(&manifest_path) {
                    let now = chrono::Utc::now();
                    let dur = now.signed_duration_since(manifest.created_at);
                    let age = if dur.num_hours() > 0 {
                        format!("{}h ago", dur.num_hours())
                    } else {
                        format!("{}m ago", dur.num_minutes())
                    };
                    format!(
                        "skill: {}, agent: {}, {age}",
                        manifest.skill_name, manifest.agent
                    )
                } else {
                    "no manifest data".into()
                }
            } else {
                "no manifest".into()
            };
            eprintln!("  {}. {short_id} ({detail})", i + 1);
        }

        eprint!("Clean up? [Y/n] ");
        let mut input = String::new();
        std::io::BufRead::read_line(&mut std::io::stdin().lock(), &mut input)
            .map_err(|e| SkillxError::Session(format!("failed to read input: {e}")))?;
        let input = input.trim().to_lowercase();
        if input == "n" || input == "no" {
            return Ok(vec![]);
        }
    }

    let mut orphans = Vec::new();
    for entry in orphan_entries {
        let session_id = entry.file_name().to_string_lossy().to_string();
        orphans.push(session_id.clone());

        if !interactive {
            ui::warn(&format!("Found orphaned session: {session_id}"));
        }

        if let Err(e) = cleanup_session(&entry.path()) {
            ui::warn(&format!("Failed to clean orphan {session_id}: {e}"));
        }
    }

    Ok(orphans)
}

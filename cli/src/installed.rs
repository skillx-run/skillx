use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::config::Config;
use crate::error::{Result, SkillxError};

/// Persistent state for installed skills, stored at `~/.skillx/installed.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledState {
    pub version: u32,
    pub skills: Vec<InstalledSkill>,
}

/// A single installed skill with its injections.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledSkill {
    pub name: String,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_ref: Option<String>,
    /// Exact commit SHA for the installed version (populated in v0.4+).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_commit: Option<String>,
    pub installed_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub scan_level: String,
    pub injections: Vec<Injection>,
}

/// An injection of a skill into a specific agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Injection {
    pub agent: String,
    pub scope: String,
    pub path: String,
    pub files: Vec<InjectedFileRecord>,
}

/// A record of an injected file with its SHA256 hash.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectedFileRecord {
    pub relative: String,
    pub sha256: String,
}

impl Default for InstalledState {
    fn default() -> Self {
        InstalledState {
            version: 1,
            skills: Vec::new(),
        }
    }
}

impl InstalledState {
    /// Path to `~/.skillx/installed.json`.
    pub fn file_path() -> Result<PathBuf> {
        Ok(Config::base_dir()?.join("installed.json"))
    }

    /// Load from `~/.skillx/installed.json`. Returns empty state if file doesn't exist.
    pub fn load() -> Result<Self> {
        let path = Self::file_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(&path).map_err(|e| {
            SkillxError::Install(format!("failed to read installed.json: {e}"))
        })?;
        let state: InstalledState = serde_json::from_str(&content).map_err(|e| {
            SkillxError::Install(format!("failed to parse installed.json: {e}"))
        })?;
        // Version check for future format migrations
        if state.version > 1 {
            return Err(SkillxError::Install(format!(
                "installed.json version {} is newer than supported (1). Please upgrade skillx.",
                state.version
            )));
        }
        Ok(state)
    }

    /// Save to `~/.skillx/installed.json` atomically. Creates parent directory if needed.
    ///
    /// Uses write-to-temp + rename to avoid corruption from concurrent access or crashes.
    pub fn save(&self) -> Result<()> {
        let path = Self::file_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                SkillxError::Install(format!(
                    "failed to create directory {}: {e}",
                    parent.display()
                ))
            })?;
        }
        let json = serde_json::to_string_pretty(self).map_err(|e| {
            SkillxError::Install(format!("failed to serialize installed.json: {e}"))
        })?;
        // Atomic write: write to temp file then rename
        let tmp_path = path.with_extension("json.tmp");
        std::fs::write(&tmp_path, &json).map_err(|e| {
            SkillxError::Install(format!("failed to write installed.json.tmp: {e}"))
        })?;
        std::fs::rename(&tmp_path, &path).map_err(|e| {
            // Fallback: if rename fails (cross-device), try direct write
            let _ = std::fs::remove_file(&tmp_path);
            SkillxError::Install(format!("failed to save installed.json: {e}"))
        })?;
        Ok(())
    }

    /// Find an installed skill by name.
    pub fn find_skill(&self, name: &str) -> Option<&InstalledSkill> {
        self.skills.iter().find(|s| s.name == name)
    }

    /// Find an installed skill by name (mutable).
    pub fn find_skill_mut(&mut self, name: &str) -> Option<&mut InstalledSkill> {
        self.skills.iter_mut().find(|s| s.name == name)
    }

    /// Add or replace a skill (matched by name).
    ///
    /// **Warning**: If the skill already exists, it is completely replaced,
    /// including all existing injections. Use `find_skill_mut()` to selectively
    /// update individual fields or injections without losing others.
    pub fn add_or_update_skill(&mut self, skill: InstalledSkill) {
        if let Some(existing) = self.skills.iter_mut().find(|s| s.name == skill.name) {
            *existing = skill;
        } else {
            self.skills.push(skill);
        }
    }

    /// Remove a skill entirely. Returns the removed skill if found.
    pub fn remove_skill(&mut self, name: &str) -> Option<InstalledSkill> {
        let pos = self.skills.iter().position(|s| s.name == name)?;
        Some(self.skills.remove(pos))
    }

    /// Remove a specific agent injection from a skill.
    /// If the skill has no remaining injections, remove the entire skill entry.
    pub fn remove_injection(&mut self, skill_name: &str, agent_name: &str) {
        if let Some(skill) = self.skills.iter_mut().find(|s| s.name == skill_name) {
            skill.injections.retain(|inj| inj.agent != agent_name);
        }
        // Remove skill entirely if no injections remain
        self.skills
            .retain(|s| s.name != skill_name || !s.injections.is_empty());
    }

    /// Check if a skill is installed.
    pub fn is_installed(&self, name: &str) -> bool {
        self.find_skill(name).is_some()
    }
}

/// Recursively collect (relative_path, sha256) pairs for all files in a directory.
/// Used for comparing installed vs fetched skill content.
pub fn collect_file_hashes(
    dir: &std::path::Path,
) -> std::result::Result<std::collections::BTreeSet<(String, String)>, std::io::Error> {
    let mut result = std::collections::BTreeSet::new();
    collect_file_hashes_inner(dir, dir, &mut result)?;
    Ok(result)
}

fn collect_file_hashes_inner(
    current: &std::path::Path,
    root: &std::path::Path,
    result: &mut std::collections::BTreeSet<(String, String)>,
) -> std::result::Result<(), std::io::Error> {
    use sha2::{Digest, Sha256};

    for entry in std::fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_file_hashes_inner(&path, root, result)?;
        } else {
            let relative = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();
            let content = std::fs::read(&path)?;
            let mut hasher = Sha256::new();
            hasher.update(&content);
            let sha256 = format!("{:x}", hasher.finalize());
            result.insert((relative, sha256));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_skill(name: &str, agent: &str) -> InstalledSkill {
        InstalledSkill {
            name: name.to_string(),
            source: format!("github:org/{name}"),
            resolved_ref: None,
            resolved_commit: None,
            installed_at: Utc::now(),
            updated_at: Utc::now(),
            scan_level: "pass".to_string(),
            injections: vec![Injection {
                agent: agent.to_string(),
                scope: "global".to_string(),
                path: format!("/path/to/{name}"),
                files: vec![InjectedFileRecord {
                    relative: "SKILL.md".to_string(),
                    sha256: "abc123".to_string(),
                }],
            }],
        }
    }

    #[test]
    fn test_default_state() {
        let state = InstalledState::default();
        assert_eq!(state.version, 1);
        assert!(state.skills.is_empty());
    }

    #[test]
    fn test_add_and_find() {
        let mut state = InstalledState::default();
        state.add_or_update_skill(make_skill("pdf", "claude-code"));

        assert!(state.find_skill("pdf").is_some());
        assert!(state.find_skill("other").is_none());
        assert!(state.is_installed("pdf"));
        assert!(!state.is_installed("other"));
    }

    #[test]
    fn test_add_or_update_replaces() {
        let mut state = InstalledState::default();
        state.add_or_update_skill(make_skill("pdf", "claude-code"));
        assert_eq!(state.skills.len(), 1);

        // Update with different agent
        state.add_or_update_skill(make_skill("pdf", "cursor"));
        assert_eq!(state.skills.len(), 1);
        assert_eq!(state.skills[0].injections[0].agent, "cursor");
    }

    #[test]
    fn test_remove_skill() {
        let mut state = InstalledState::default();
        state.add_or_update_skill(make_skill("pdf", "claude-code"));
        state.add_or_update_skill(make_skill("review", "cursor"));

        let removed = state.remove_skill("pdf");
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().name, "pdf");
        assert_eq!(state.skills.len(), 1);
        assert_eq!(state.skills[0].name, "review");

        assert!(state.remove_skill("nonexistent").is_none());
    }

    #[test]
    fn test_remove_injection_partial() {
        let mut state = InstalledState::default();
        let mut skill = make_skill("pdf", "claude-code");
        skill.injections.push(Injection {
            agent: "cursor".to_string(),
            scope: "global".to_string(),
            path: "/path/to/pdf-cursor".to_string(),
            files: vec![InjectedFileRecord {
                relative: "SKILL.md".to_string(),
                sha256: "abc123".to_string(),
            }],
        });
        state.add_or_update_skill(skill);

        // Remove only cursor injection
        state.remove_injection("pdf", "cursor");

        // Skill still exists with claude-code injection
        let skill = state.find_skill("pdf").unwrap();
        assert_eq!(skill.injections.len(), 1);
        assert_eq!(skill.injections[0].agent, "claude-code");
    }

    #[test]
    fn test_remove_injection_complete() {
        let mut state = InstalledState::default();
        state.add_or_update_skill(make_skill("pdf", "claude-code"));

        // Remove the only injection -> skill should be removed entirely
        state.remove_injection("pdf", "claude-code");
        assert!(!state.is_installed("pdf"));
        assert!(state.skills.is_empty());
    }

    #[test]
    fn test_remove_injection_nonexistent() {
        let mut state = InstalledState::default();
        state.add_or_update_skill(make_skill("pdf", "claude-code"));

        // Removing nonexistent agent doesn't affect anything
        state.remove_injection("pdf", "nonexistent");
        assert!(state.is_installed("pdf"));
        assert_eq!(state.find_skill("pdf").unwrap().injections.len(), 1);
    }

    #[test]
    fn test_json_roundtrip() {
        let mut state = InstalledState::default();
        state.add_or_update_skill(make_skill("pdf", "claude-code"));
        state.add_or_update_skill(make_skill("review", "cursor"));

        let json = serde_json::to_string_pretty(&state).unwrap();
        let loaded: InstalledState = serde_json::from_str(&json).unwrap();

        assert_eq!(loaded.version, 1);
        assert_eq!(loaded.skills.len(), 2);
        assert_eq!(loaded.skills[0].name, "pdf");
        assert_eq!(loaded.skills[1].name, "review");
        assert_eq!(loaded.skills[0].injections[0].agent, "claude-code");
        assert_eq!(loaded.skills[0].injections[0].files[0].relative, "SKILL.md");
        assert_eq!(loaded.skills[0].injections[0].files[0].sha256, "abc123");
    }

    #[test]
    fn test_multi_agent_single_skill() {
        let mut state = InstalledState::default();
        let mut skill = make_skill("pdf", "claude-code");
        skill.injections.push(Injection {
            agent: "cursor".to_string(),
            scope: "project".to_string(),
            path: "/cursor/path".to_string(),
            files: vec![InjectedFileRecord {
                relative: "SKILL.md".to_string(),
                sha256: "def456".to_string(),
            }],
        });
        state.add_or_update_skill(skill);

        let skill = state.find_skill("pdf").unwrap();
        assert_eq!(skill.injections.len(), 2);
        assert_eq!(skill.injections[0].agent, "claude-code");
        assert_eq!(skill.injections[1].agent, "cursor");
    }

    #[test]
    fn test_find_skill_mut() {
        let mut state = InstalledState::default();
        state.add_or_update_skill(make_skill("pdf", "claude-code"));

        let skill = state.find_skill_mut("pdf").unwrap();
        skill.source = "github:new/source".to_string();

        assert_eq!(state.find_skill("pdf").unwrap().source, "github:new/source");
    }
}

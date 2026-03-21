use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{Result, SkillxError};

/// Project-level configuration loaded from `skillx.toml`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ProjectConfig {
    pub project: ProjectInfo,
    pub skills: Vec<SkillEntry>,
    pub defaults: ProjectDefaults,
}

/// Project metadata.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ProjectInfo {
    pub name: Option<String>,
}

/// A skill entry in `[[skills]]`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillEntry {
    pub source: String,
    pub agent: Option<String>,
    pub scope: Option<String>,
    pub skip_scan: Option<bool>,
}

/// Default values for skill entries.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ProjectDefaults {
    pub agent: Option<String>,
    pub scope: Option<String>,
    pub skip_scan: Option<bool>,
}

impl ProjectConfig {
    /// Load a `skillx.toml` from the given directory.
    ///
    /// Returns `Ok(None)` if the file does not exist.
    pub fn load(dir: &Path) -> Result<Option<ProjectConfig>> {
        let path = dir.join("skillx.toml");
        if !path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&path).map_err(|e| {
            SkillxError::ProjectConfig(format!("failed to read skillx.toml: {e}"))
        })?;

        let config: ProjectConfig = toml::from_str(&content).map_err(|e| {
            SkillxError::ProjectConfig(format!("failed to parse skillx.toml: {e}"))
        })?;

        // Validate: source fields must not be empty
        for (i, entry) in config.skills.iter().enumerate() {
            if entry.source.trim().is_empty() {
                return Err(SkillxError::ProjectConfig(format!(
                    "skills[{i}].source must not be empty"
                )));
            }
        }

        Ok(Some(config))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_load_valid_skillx_toml() {
        let dir = tempfile::tempdir().unwrap();
        let toml_content = r#"
[project]
name = "my-project"

[[skills]]
source = "github:anthropics/skills/pdf-processing"
agent = "claude-code"
scope = "project"

[[skills]]
source = "https://github.com/owner/repo"
skip_scan = true

[defaults]
agent = "claude-code"
scope = "project"
skip_scan = false
"#;
        fs::write(dir.path().join("skillx.toml"), toml_content).unwrap();
        let config = ProjectConfig::load(dir.path()).unwrap().unwrap();

        assert_eq!(config.project.name.as_deref(), Some("my-project"));
        assert_eq!(config.skills.len(), 2);
        assert_eq!(config.skills[0].source, "github:anthropics/skills/pdf-processing");
        assert_eq!(config.skills[0].agent.as_deref(), Some("claude-code"));
        assert_eq!(config.skills[0].scope.as_deref(), Some("project"));
        assert_eq!(config.skills[1].source, "https://github.com/owner/repo");
        assert_eq!(config.skills[1].skip_scan, Some(true));
        assert_eq!(config.defaults.agent.as_deref(), Some("claude-code"));
        assert_eq!(config.defaults.scope.as_deref(), Some("project"));
        assert_eq!(config.defaults.skip_scan, Some(false));
    }

    #[test]
    fn test_load_nonexistent_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let config = ProjectConfig::load(dir.path()).unwrap();
        assert!(config.is_none());
    }

    #[test]
    fn test_load_invalid_toml_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("skillx.toml"), "not valid {{toml").unwrap();
        let result = ProjectConfig::load(dir.path());
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("project config error"));
    }

    #[test]
    fn test_load_minimal_defaults_only() {
        let dir = tempfile::tempdir().unwrap();
        let toml_content = r#"
[defaults]
agent = "cursor"
"#;
        fs::write(dir.path().join("skillx.toml"), toml_content).unwrap();
        let config = ProjectConfig::load(dir.path()).unwrap().unwrap();

        assert!(config.project.name.is_none());
        assert!(config.skills.is_empty());
        assert_eq!(config.defaults.agent.as_deref(), Some("cursor"));
    }

    #[test]
    fn test_load_empty_source_validation() {
        let dir = tempfile::tempdir().unwrap();
        let toml_content = r#"
[[skills]]
source = ""
"#;
        fs::write(dir.path().join("skillx.toml"), toml_content).unwrap();
        let result = ProjectConfig::load(dir.path());
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("source must not be empty"));
    }
}

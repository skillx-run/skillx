use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{Result, SkillxError};

/// Value for a skill entry: either a simple source string or a detailed object.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum SkillValue {
    /// Short form: `name = "source-string"`
    Simple(String),
    /// Expanded form: `name = { source = "...", scope = "...", skip_scan = true }`
    Detailed {
        source: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        scope: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        skip_scan: Option<bool>,
    },
}

impl SkillValue {
    pub fn source(&self) -> &str {
        match self {
            SkillValue::Simple(s) => s,
            SkillValue::Detailed { source, .. } => source,
        }
    }

    pub fn scope(&self) -> Option<&str> {
        match self {
            SkillValue::Simple(_) => None,
            SkillValue::Detailed { scope, .. } => scope.as_deref(),
        }
    }

    pub fn skip_scan(&self) -> Option<bool> {
        match self {
            SkillValue::Simple(_) => None,
            SkillValue::Detailed { skip_scan, .. } => *skip_scan,
        }
    }
}

/// The `[skills]` section, with optional `[skills.dev]` sub-table.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillsSection {
    /// Regular skill entries (via `#[serde(flatten)]`)
    #[serde(flatten)]
    pub entries: BTreeMap<String, SkillValue>,

    /// Dev-only skills from `[skills.dev]`
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub dev: BTreeMap<String, SkillValue>,
}

/// Project metadata under `[project]`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ProjectInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Agent settings under `[agent]`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AgentSettings {
    /// Preferred agent name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred: Option<String>,
    /// Default injection scope ("project" | "global")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// List of agents to inject into during install
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub targets: Vec<String>,
}

/// Project-level configuration loaded from `skillx.toml`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ProjectConfig {
    pub project: ProjectInfo,
    pub agent: AgentSettings,
    pub skills: SkillsSection,
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

        let content = std::fs::read_to_string(&path)
            .map_err(|e| SkillxError::ProjectConfig(format!("failed to read skillx.toml: {e}")))?;

        // Detect old [[skills]] array format
        if content.contains("[[skills]]") {
            return Err(SkillxError::ProjectConfig(
                "skillx.toml uses the deprecated [[skills]] array format. \
                 Please migrate to the new [skills] table format:\n\
                 \n\
                 [skills]\n\
                 my-skill = \"github:owner/repo/path\"\n\
                 another = { source = \"github:org/skills/name\", scope = \"project\" }\n"
                    .to_string(),
            ));
        }

        let config: ProjectConfig = toml::from_str(&content)
            .map_err(|e| SkillxError::ProjectConfig(format!("failed to parse skillx.toml: {e}")))?;

        // Validate: source fields must not be empty
        for (name, value) in &config.skills.entries {
            if value.source().trim().is_empty() {
                return Err(SkillxError::ProjectConfig(format!(
                    "skills.{name}.source must not be empty"
                )));
            }
        }
        for (name, value) in &config.skills.dev {
            if value.source().trim().is_empty() {
                return Err(SkillxError::ProjectConfig(format!(
                    "skills.dev.{name}.source must not be empty"
                )));
            }
        }

        Ok(Some(config))
    }

    /// Save the config to `skillx.toml` in the given directory.
    pub fn save(&self, dir: &Path) -> Result<()> {
        let path = dir.join("skillx.toml");
        let content = toml::to_string_pretty(self).map_err(|e| {
            SkillxError::ProjectConfig(format!("failed to serialize skillx.toml: {e}"))
        })?;
        std::fs::write(&path, content)
            .map_err(|e| SkillxError::ProjectConfig(format!("failed to write skillx.toml: {e}")))?;
        Ok(())
    }

    /// Add a skill entry. If `dev` is true, adds to `[skills.dev]`.
    pub fn add_skill(&mut self, name: &str, source: &str, dev: bool) {
        let value = SkillValue::Simple(source.to_string());
        if dev {
            self.skills.dev.insert(name.to_string(), value);
        } else {
            self.skills.entries.insert(name.to_string(), value);
        }
    }

    /// Remove a skill by name from both skills and skills.dev. Returns true if found.
    pub fn remove_skill(&mut self, name: &str) -> bool {
        let a = self.skills.entries.remove(name).is_some();
        let b = self.skills.dev.remove(name).is_some();
        a || b
    }

    /// Create a default empty skillx.toml in the given directory.
    pub fn create_default(dir: &Path) -> Result<()> {
        let config = ProjectConfig {
            project: ProjectInfo {
                name: None,
                description: None,
            },
            agent: AgentSettings::default(),
            skills: SkillsSection::default(),
        };
        config.save(dir)
    }

    /// Create a skillx.toml from a list of installed skills.
    /// Each tuple is (name, source).
    pub fn create_from_installed(dir: &Path, skills: &[(String, String)]) -> Result<()> {
        let mut config = ProjectConfig::default();
        for (name, source) in skills {
            config.add_skill(name, source, false);
        }
        config.save(dir)
    }

    /// Return all skills as (name, value, is_dev) tuples.
    pub fn all_skills(&self) -> Vec<(String, &SkillValue, bool)> {
        let mut result: Vec<(String, &SkillValue, bool)> = Vec::new();
        for (name, value) in &self.skills.entries {
            result.push((name.clone(), value, false));
        }
        for (name, value) in &self.skills.dev {
            result.push((name.clone(), value, true));
        }
        result
    }

    /// Update the source string of an existing skill entry.
    /// For Detailed entries, only updates `source`, preserving scope and skip_scan.
    /// Returns true if found and the value actually changed.
    pub fn update_skill_source(&mut self, name: &str, new_source: &str) -> bool {
        for map in [&mut self.skills.entries, &mut self.skills.dev] {
            if let Some(value) = map.get_mut(name) {
                // Skip if source is already the same
                if value.source() == new_source {
                    return false;
                }
                match value {
                    SkillValue::Simple(_) => {
                        *value = SkillValue::Simple(new_source.to_string());
                    }
                    SkillValue::Detailed { source, .. } => {
                        *source = new_source.to_string();
                    }
                }
                return true;
            }
        }
        false
    }

    /// Check if any skills are defined.
    pub fn has_skills(&self) -> bool {
        !self.skills.entries.is_empty() || !self.skills.dev.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_load_simple_string_skills() {
        let dir = tempfile::tempdir().unwrap();
        let toml_content = r#"
[project]
name = "my-project"

[skills]
pdf-processing = "github:anthropics/skills/pdf-processing"
code-review = "github:org/skills/code-review"
"#;
        fs::write(dir.path().join("skillx.toml"), toml_content).unwrap();
        let config = ProjectConfig::load(dir.path()).unwrap().unwrap();

        assert_eq!(config.project.name.as_deref(), Some("my-project"));
        assert_eq!(config.skills.entries.len(), 2);
        assert_eq!(
            config.skills.entries["pdf-processing"].source(),
            "github:anthropics/skills/pdf-processing"
        );
        assert_eq!(
            config.skills.entries["code-review"].source(),
            "github:org/skills/code-review"
        );
    }

    #[test]
    fn test_load_detailed_object_skills() {
        let dir = tempfile::tempdir().unwrap();
        let toml_content = r#"
[skills]
code-review = { source = "github:org/skills/cr@v2.1", scope = "project", skip_scan = true }
"#;
        fs::write(dir.path().join("skillx.toml"), toml_content).unwrap();
        let config = ProjectConfig::load(dir.path()).unwrap().unwrap();

        let cr = &config.skills.entries["code-review"];
        assert_eq!(cr.source(), "github:org/skills/cr@v2.1");
        assert_eq!(cr.scope(), Some("project"));
        assert_eq!(cr.skip_scan(), Some(true));
    }

    #[test]
    fn test_load_mixed_simple_and_detailed() {
        let dir = tempfile::tempdir().unwrap();
        let toml_content = r#"
[skills]
pdf = "github:anthropics/skills/pdf"
review = { source = "github:org/skills/cr", scope = "project" }
"#;
        fs::write(dir.path().join("skillx.toml"), toml_content).unwrap();
        let config = ProjectConfig::load(dir.path()).unwrap().unwrap();

        assert_eq!(config.skills.entries.len(), 2);
        assert!(matches!(
            config.skills.entries["pdf"],
            SkillValue::Simple(_)
        ));
        assert!(matches!(
            config.skills.entries["review"],
            SkillValue::Detailed { .. }
        ));
    }

    #[test]
    fn test_load_skills_dev() {
        let dir = tempfile::tempdir().unwrap();
        let toml_content = r#"
[skills]
pdf = "github:anthropics/skills/pdf"

[skills.dev]
testing = "github:org/skills/testing"
debug = { source = "github:org/skills/debug", scope = "project" }
"#;
        fs::write(dir.path().join("skillx.toml"), toml_content).unwrap();
        let config = ProjectConfig::load(dir.path()).unwrap().unwrap();

        assert_eq!(config.skills.entries.len(), 1);
        assert_eq!(config.skills.dev.len(), 2);
        assert_eq!(
            config.skills.dev["testing"].source(),
            "github:org/skills/testing"
        );
        assert_eq!(
            config.skills.dev["debug"].source(),
            "github:org/skills/debug"
        );
    }

    #[test]
    fn test_load_agent_settings() {
        let dir = tempfile::tempdir().unwrap();
        let toml_content = r#"
[agent]
preferred = "claude-code"
scope = "project"
targets = ["claude-code", "cursor"]
"#;
        fs::write(dir.path().join("skillx.toml"), toml_content).unwrap();
        let config = ProjectConfig::load(dir.path()).unwrap().unwrap();

        assert_eq!(config.agent.preferred.as_deref(), Some("claude-code"));
        assert_eq!(config.agent.scope.as_deref(), Some("project"));
        assert_eq!(config.agent.targets, vec!["claude-code", "cursor"]);
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
    fn test_load_old_format_detected() {
        let dir = tempfile::tempdir().unwrap();
        let toml_content = r#"
[[skills]]
source = "github:anthropics/skills/pdf-processing"
"#;
        fs::write(dir.path().join("skillx.toml"), toml_content).unwrap();
        let result = ProjectConfig::load(dir.path());
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("deprecated"));
        assert!(err.contains("[[skills]]"));
    }

    #[test]
    fn test_load_empty_source_validation() {
        let dir = tempfile::tempdir().unwrap();
        let toml_content = r#"
[skills]
bad = ""
"#;
        fs::write(dir.path().join("skillx.toml"), toml_content).unwrap();
        let result = ProjectConfig::load(dir.path());
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("source must not be empty"));
    }

    #[test]
    fn test_load_empty_source_dev_validation() {
        let dir = tempfile::tempdir().unwrap();
        let toml_content = r#"
[skills.dev]
bad = { source = "  " }
"#;
        fs::write(dir.path().join("skillx.toml"), toml_content).unwrap();
        let result = ProjectConfig::load(dir.path());
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("source must not be empty"));
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let mut config = ProjectConfig::default();
        config.project.name = Some("roundtrip-test".to_string());
        config.agent.preferred = Some("claude-code".to_string());
        config.skills.entries.insert(
            "pdf".to_string(),
            SkillValue::Simple("github:org/pdf".to_string()),
        );
        config.skills.entries.insert(
            "review".to_string(),
            SkillValue::Detailed {
                source: "github:org/review".to_string(),
                scope: Some("project".to_string()),
                skip_scan: None,
            },
        );
        config.skills.dev.insert(
            "testing".to_string(),
            SkillValue::Simple("github:org/testing".to_string()),
        );

        config.save(dir.path()).unwrap();
        let loaded = ProjectConfig::load(dir.path()).unwrap().unwrap();

        assert_eq!(loaded.project.name.as_deref(), Some("roundtrip-test"));
        assert_eq!(loaded.agent.preferred.as_deref(), Some("claude-code"));
        assert_eq!(loaded.skills.entries.len(), 2);
        assert_eq!(loaded.skills.entries["pdf"].source(), "github:org/pdf");
        assert_eq!(
            loaded.skills.entries["review"].source(),
            "github:org/review"
        );
        assert_eq!(loaded.skills.entries["review"].scope(), Some("project"));
        assert_eq!(loaded.skills.dev.len(), 1);
        assert_eq!(loaded.skills.dev["testing"].source(), "github:org/testing");
    }

    #[test]
    fn test_add_skill() {
        let mut config = ProjectConfig::default();
        config.add_skill("pdf", "github:org/pdf", false);
        config.add_skill("testing", "github:org/testing", true);

        assert_eq!(config.skills.entries.len(), 1);
        assert_eq!(config.skills.entries["pdf"].source(), "github:org/pdf");
        assert_eq!(config.skills.dev.len(), 1);
        assert_eq!(config.skills.dev["testing"].source(), "github:org/testing");
    }

    #[test]
    fn test_remove_skill() {
        let mut config = ProjectConfig::default();
        config.add_skill("pdf", "github:org/pdf", false);
        config.add_skill("testing", "github:org/testing", true);

        assert!(config.remove_skill("pdf"));
        assert!(config.skills.entries.is_empty());

        assert!(config.remove_skill("testing"));
        assert!(config.skills.dev.is_empty());

        assert!(!config.remove_skill("nonexistent"));
    }

    #[test]
    fn test_all_skills() {
        let mut config = ProjectConfig::default();
        config.add_skill("pdf", "github:org/pdf", false);
        config.add_skill("review", "github:org/review", false);
        config.add_skill("testing", "github:org/testing", true);

        let all = config.all_skills();
        assert_eq!(all.len(), 3);

        // BTreeMap is sorted by key
        let (name, val, is_dev) = &all[0];
        assert_eq!(name, "pdf");
        assert_eq!(val.source(), "github:org/pdf");
        assert!(!is_dev);

        let (name, val, is_dev) = &all[1];
        assert_eq!(name, "review");
        assert_eq!(val.source(), "github:org/review");
        assert!(!is_dev);

        let (name, val, is_dev) = &all[2];
        assert_eq!(name, "testing");
        assert_eq!(val.source(), "github:org/testing");
        assert!(is_dev);
    }

    #[test]
    fn test_has_skills() {
        let mut config = ProjectConfig::default();
        assert!(!config.has_skills());

        config.add_skill("pdf", "github:org/pdf", false);
        assert!(config.has_skills());
    }

    #[test]
    fn test_create_default() {
        let dir = tempfile::tempdir().unwrap();
        ProjectConfig::create_default(dir.path()).unwrap();

        let loaded = ProjectConfig::load(dir.path()).unwrap().unwrap();
        assert!(loaded.project.name.is_none());
        assert!(!loaded.has_skills());
    }

    #[test]
    fn test_create_from_installed() {
        let dir = tempfile::tempdir().unwrap();
        let skills = vec![
            ("pdf".to_string(), "github:org/pdf".to_string()),
            ("review".to_string(), "github:org/review".to_string()),
        ];
        ProjectConfig::create_from_installed(dir.path(), &skills).unwrap();

        let loaded = ProjectConfig::load(dir.path()).unwrap().unwrap();
        assert_eq!(loaded.skills.entries.len(), 2);
        assert_eq!(loaded.skills.entries["pdf"].source(), "github:org/pdf");
        assert_eq!(
            loaded.skills.entries["review"].source(),
            "github:org/review"
        );
    }

    #[test]
    fn test_skill_value_accessors() {
        let simple = SkillValue::Simple("source".to_string());
        assert_eq!(simple.source(), "source");
        assert_eq!(simple.scope(), None);
        assert_eq!(simple.skip_scan(), None);

        let detailed = SkillValue::Detailed {
            source: "src".to_string(),
            scope: Some("project".to_string()),
            skip_scan: Some(true),
        };
        assert_eq!(detailed.source(), "src");
        assert_eq!(detailed.scope(), Some("project"));
        assert_eq!(detailed.skip_scan(), Some(true));
    }

    #[test]
    fn test_update_skill_source_simple() {
        let mut config = ProjectConfig::default();
        config.add_skill("pdf", "github:org/pdf@v1.0", false);

        assert!(config.update_skill_source("pdf", "github:org/pdf@v1.1"));
        assert_eq!(config.skills.entries["pdf"].source(), "github:org/pdf@v1.1");
    }

    #[test]
    fn test_update_skill_source_detailed_preserves_scope() {
        let mut config = ProjectConfig::default();
        config.skills.entries.insert(
            "review".to_string(),
            SkillValue::Detailed {
                source: "github:org/review@v1.0".to_string(),
                scope: Some("project".to_string()),
                skip_scan: Some(true),
            },
        );

        assert!(config.update_skill_source("review", "github:org/review@v2.0"));
        let val = &config.skills.entries["review"];
        assert_eq!(val.source(), "github:org/review@v2.0");
        assert_eq!(val.scope(), Some("project"));
        assert_eq!(val.skip_scan(), Some(true));
    }

    #[test]
    fn test_update_skill_source_same_value_returns_false() {
        let mut config = ProjectConfig::default();
        config.add_skill("pdf", "github:org/pdf@v1.0", false);

        // Same source — should return false (no change)
        assert!(!config.update_skill_source("pdf", "github:org/pdf@v1.0"));
        assert_eq!(config.skills.entries["pdf"].source(), "github:org/pdf@v1.0");
    }

    #[test]
    fn test_update_skill_source_not_found() {
        let mut config = ProjectConfig::default();
        config.add_skill("pdf", "github:org/pdf", false);

        assert!(!config.update_skill_source("nonexistent", "github:org/new"));
    }

    #[test]
    fn test_update_skill_source_in_dev() {
        let mut config = ProjectConfig::default();
        config.add_skill("testing", "github:org/testing@v1.0", true);

        assert!(config.update_skill_source("testing", "github:org/testing@v2.0"));
        assert_eq!(
            config.skills.dev["testing"].source(),
            "github:org/testing@v2.0"
        );
    }

    #[test]
    fn test_minimal_agent_only() {
        let dir = tempfile::tempdir().unwrap();
        let toml_content = r#"
[agent]
preferred = "cursor"
"#;
        fs::write(dir.path().join("skillx.toml"), toml_content).unwrap();
        let config = ProjectConfig::load(dir.path()).unwrap().unwrap();

        assert!(config.project.name.is_none());
        assert!(!config.has_skills());
        assert_eq!(config.agent.preferred.as_deref(), Some("cursor"));
    }
}

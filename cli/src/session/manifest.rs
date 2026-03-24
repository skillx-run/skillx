use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::error::{Result, SkillxError};
use crate::scanner::ScanReport;
use crate::session::inject::{InjectedRecord, InjectionType};

/// Manifest tracking all injected files for a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub session_id: String,
    pub skill_name: String,
    pub source: String,
    pub agent: String,
    pub lifecycle_mode: String,
    pub scope: String,
    pub created_at: DateTime<Utc>,
    pub injected_files: Vec<InjectedFile>,
    pub injected_attachments: Vec<InjectedAttachment>,
    pub scan_result: Option<ScanReport>,
}

/// A file that was injected into the agent's skill directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectedFile {
    pub path: String,
    pub sha256: String,
    /// How this file was injected — determines cleanup strategy.
    #[serde(default)]
    pub injection_type: InjectionType,
}

/// An attachment that was copied alongside the skill.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectedAttachment {
    pub original: String,
    pub copied_to: String,
}

impl Manifest {
    /// Create a new manifest.
    pub fn new(
        session_id: &str,
        skill_name: &str,
        source: &str,
        agent: &str,
        lifecycle_mode: &str,
        scope: &str,
    ) -> Self {
        Manifest {
            session_id: session_id.to_string(),
            skill_name: skill_name.to_string(),
            source: source.to_string(),
            agent: agent.to_string(),
            lifecycle_mode: lifecycle_mode.to_string(),
            scope: scope.to_string(),
            created_at: Utc::now(),
            injected_files: Vec::new(),
            injected_attachments: Vec::new(),
            scan_result: None,
        }
    }

    /// Save manifest to a path.
    pub fn save(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| SkillxError::Session(format!("failed to serialize manifest: {e}")))?;
        std::fs::write(path, json)
            .map_err(|e| SkillxError::Session(format!("failed to write manifest: {e}")))?;
        Ok(())
    }

    /// Load manifest from a path.
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| SkillxError::Session(format!("failed to read manifest: {e}")))?;
        let manifest: Manifest = serde_json::from_str(&content)
            .map_err(|e| SkillxError::Session(format!("failed to parse manifest: {e}")))?;
        Ok(manifest)
    }

    /// Add an injected file record.
    pub fn add_file(&mut self, path: String, sha256: String) {
        self.injected_files.push(InjectedFile {
            path,
            sha256,
            injection_type: InjectionType::CopiedFile,
        });
    }

    /// Add an injected file from an InjectedRecord (includes injection_type).
    pub fn add_record(&mut self, record: &InjectedRecord) {
        self.injected_files.push(InjectedFile {
            path: record.path.clone(),
            sha256: record.sha256.clone(),
            injection_type: record.injection_type.clone(),
        });
    }

    /// Add an injected attachment record.
    pub fn add_attachment(&mut self, original: String, copied_to: String) {
        self.injected_attachments.push(InjectedAttachment {
            original,
            copied_to,
        });
    }

    /// Manifest file path within a session directory.
    pub fn manifest_path(session_dir: &Path) -> PathBuf {
        session_dir.join("manifest.json")
    }
}

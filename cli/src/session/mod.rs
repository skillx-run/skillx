pub mod cleanup;
pub mod inject;
pub mod manifest;

use chrono::{DateTime, Utc};

use crate::config::Config;
use crate::error::{Result, SkillxError};

/// An active run session.
#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub skill_name: String,
    pub created_at: DateTime<Utc>,
}

impl Session {
    /// Create a new session with a unique ID.
    pub fn new(skill_name: &str) -> Self {
        let id = uuid::Uuid::new_v4().to_string()[..8].to_string();
        Session {
            id,
            skill_name: skill_name.to_string(),
            created_at: Utc::now(),
        }
    }

    /// Session directory: `~/.skillx/active/<session-id>/`
    pub fn session_dir(&self) -> Result<std::path::PathBuf> {
        Ok(Config::active_dir()?.join(&self.id))
    }

    /// Create the session directory structure.
    pub fn create_dirs(&self) -> Result<()> {
        let dir = self.session_dir()?;
        std::fs::create_dir_all(dir.join("skill-files"))
            .map_err(|e| SkillxError::Session(format!("failed to create session dir: {e}")))?;
        std::fs::create_dir_all(dir.join("attachments"))
            .map_err(|e| SkillxError::Session(format!("failed to create attachments dir: {e}")))?;
        Ok(())
    }

    /// List active sessions by scanning `~/.skillx/active/`.
    pub fn list_active() -> Result<Vec<String>> {
        let active_dir = Config::active_dir()?;
        if !active_dir.exists() {
            return Ok(vec![]);
        }

        let mut sessions = Vec::new();
        let entries = std::fs::read_dir(&active_dir)
            .map_err(|e| SkillxError::Session(format!("failed to read active dir: {e}")))?;

        for entry in entries {
            let entry = entry
                .map_err(|e| SkillxError::Session(format!("dir entry error: {e}")))?;
            if entry.path().is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    sessions.push(name.to_string());
                }
            }
        }

        Ok(sessions)
    }
}

use async_trait::async_trait;
use std::path::PathBuf;

use super::{AgentAdapter, DetectResult, LaunchConfig, LifecycleMode, SessionHandle};
use crate::error::{Result, SkillxError};
use crate::types::Scope;

pub struct CodexAdapter;

#[async_trait]
impl AgentAdapter for CodexAdapter {
    fn name(&self) -> &str {
        "codex"
    }

    fn display_name(&self) -> &str {
        "OpenAI Codex"
    }

    async fn detect(&self) -> DetectResult {
        let has_binary = which::which("codex").is_ok();
        let has_dir = dirs::home_dir()
            .map(|h| h.join(".codex").exists())
            .unwrap_or(false);

        let version = if has_binary {
            super::detect_binary_version("codex").await
        } else {
            None
        };

        DetectResult {
            name: self.name().to_string(),
            detected: has_binary || has_dir,
            version,
            info: if has_binary {
                Some("codex binary found".into())
            } else if has_dir {
                Some("~/.codex/ directory found".into())
            } else {
                None
            },
        }
    }

    fn lifecycle_mode(&self) -> LifecycleMode {
        LifecycleMode::ManagedProcess
    }

    fn supports_initial_prompt(&self) -> bool {
        true
    }

    fn supports_yolo(&self) -> bool {
        true
    }

    fn yolo_args(&self) -> Vec<&str> {
        vec!["--full-auto"]
    }

    fn inject_path(&self, skill_name: &str, scope: &Scope) -> PathBuf {
        match scope {
            Scope::Project => PathBuf::from(".agents").join("skills").join(skill_name),
            Scope::Global => super::home_dir_or_fallback()
                .join(".codex")
                .join("skills")
                .join(skill_name),
        }
    }

    async fn launch(&self, config: LaunchConfig) -> Result<SessionHandle> {
        let mut cmd = tokio::process::Command::new("codex");

        if let Some(ref prompt) = config.prompt {
            cmd.arg(prompt);
        }

        if config.yolo {
            for arg in self.yolo_args() {
                cmd.arg(arg);
            }
        }

        for arg in &config.extra_args {
            cmd.arg(arg);
        }

        let child = cmd
            .spawn()
            .map_err(|e| SkillxError::Agent(format!("failed to launch codex: {e}")))?;

        Ok(SessionHandle {
            child: Some(child),
            lifecycle_mode: self.lifecycle_mode(),
        })
    }
}

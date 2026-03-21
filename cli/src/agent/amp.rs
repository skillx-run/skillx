use async_trait::async_trait;
use std::path::PathBuf;

use super::{AgentAdapter, DetectResult, LaunchConfig, LifecycleMode, SessionHandle};
use crate::error::{Result, SkillxError};
use crate::types::Scope;

pub struct AmpAdapter;

#[async_trait]
impl AgentAdapter for AmpAdapter {
    fn name(&self) -> &str {
        "amp"
    }

    fn display_name(&self) -> &str {
        "Amp"
    }

    async fn detect(&self) -> DetectResult {
        let has_binary = which::which("amp").is_ok();
        let has_dir = dirs::home_dir()
            .map(|h| h.join(".amp").exists())
            .unwrap_or(false);

        let version = if has_binary {
            super::detect_binary_version("amp").await
        } else {
            None
        };

        DetectResult {
            name: self.name().to_string(),
            detected: has_binary || has_dir,
            version,
            info: if has_binary {
                Some("amp binary found".into())
            } else if has_dir {
                Some("~/.amp/ directory found".into())
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
        false
    }

    fn inject_path(&self, skill_name: &str, scope: &Scope) -> PathBuf {
        match scope {
            Scope::Project => PathBuf::from(".amp")
                .join("skills")
                .join(skill_name),
            Scope::Global => super::home_dir_or_fallback()
                .join(".amp")
                .join("skills")
                .join(skill_name),
        }
    }

    async fn launch(&self, config: LaunchConfig) -> Result<SessionHandle> {
        let mut cmd = tokio::process::Command::new("amp");

        if let Some(ref prompt) = config.prompt {
            cmd.arg("--prompt").arg(prompt);
        }

        for arg in &config.extra_args {
            cmd.arg(arg);
        }

        let child = cmd.spawn().map_err(|e| {
            SkillxError::Agent(format!("failed to launch amp: {e}"))
        })?;

        Ok(SessionHandle {
            child: Some(child),
            lifecycle_mode: self.lifecycle_mode(),
        })
    }
}

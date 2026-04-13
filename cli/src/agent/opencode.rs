use async_trait::async_trait;
use std::path::PathBuf;

use super::{AgentAdapter, DetectResult, LaunchConfig, LifecycleMode, SessionHandle};
use crate::error::{Result, SkillxError};
use crate::types::Scope;

pub struct OpenCodeAdapter;

#[async_trait]
impl AgentAdapter for OpenCodeAdapter {
    fn name(&self) -> &str {
        "opencode"
    }

    fn display_name(&self) -> &str {
        "OpenCode"
    }

    async fn detect(&self) -> DetectResult {
        let has_binary = which::which("opencode").is_ok();
        let has_dir = dirs::home_dir()
            .map(|h| h.join(".config").join("opencode").exists())
            .unwrap_or(false);

        let version = if has_binary {
            super::detect_binary_version("opencode").await
        } else {
            None
        };

        DetectResult {
            name: self.name().to_string(),
            detected: has_binary || has_dir,
            version,
            info: if has_binary {
                Some("opencode binary found".into())
            } else if has_dir {
                Some("~/.config/opencode/ directory found".into())
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

    fn supports_auto_approve(&self) -> bool {
        false
    }

    fn inject_path(&self, skill_name: &str, scope: &Scope) -> PathBuf {
        match scope {
            Scope::Project => PathBuf::from(".opencode").join("skills").join(skill_name),
            Scope::Global => super::home_dir_or_fallback()
                .join(".opencode")
                .join("skills")
                .join(skill_name),
        }
    }

    async fn launch(&self, config: LaunchConfig) -> Result<SessionHandle> {
        let mut cmd = tokio::process::Command::new("opencode");

        if let Some(ref prompt) = config.prompt {
            if config.print_mode {
                // Non-interactive: opencode run "prompt" (auto-approves all permissions)
                cmd.arg("run").arg(prompt);
            } else {
                // Interactive: opencode "prompt"
                cmd.arg(prompt);
            }
        }

        for arg in &config.extra_args {
            cmd.arg(arg);
        }

        let child = cmd
            .spawn()
            .map_err(|e| SkillxError::Agent(format!("failed to launch opencode: {e}")))?;

        Ok(SessionHandle {
            child: Some(child),
            lifecycle_mode: self.lifecycle_mode(),
        })
    }
}

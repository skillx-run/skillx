use async_trait::async_trait;
use std::path::PathBuf;

use super::{AgentAdapter, DetectResult, LaunchConfig, LifecycleMode, SessionHandle};
use crate::error::{Result, SkillxError};
use crate::types::Scope;

pub struct ClaudeCodeAdapter;

#[async_trait]
impl AgentAdapter for ClaudeCodeAdapter {
    fn name(&self) -> &str {
        "claude-code"
    }

    fn display_name(&self) -> &str {
        "Claude Code"
    }

    async fn detect(&self) -> DetectResult {
        let has_binary = which::which("claude").is_ok();
        let has_dir = dirs::home_dir()
            .map(|h| h.join(".claude").exists())
            .unwrap_or(false);

        DetectResult {
            name: self.name().to_string(),
            detected: has_binary || has_dir,
            version: None,
            info: if has_binary {
                Some("claude binary found".into())
            } else if has_dir {
                Some("~/.claude/ directory found".into())
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
        vec!["--dangerously-skip-permissions"]
    }

    fn inject_path(&self, skill_name: &str, scope: &Scope) -> PathBuf {
        match scope {
            Scope::Project => PathBuf::from(".claude")
                .join("skills")
                .join(skill_name),
            Scope::Global => dirs::home_dir()
                .unwrap_or_default()
                .join(".claude")
                .join("skills")
                .join(skill_name),
        }
    }

    async fn launch(&self, config: LaunchConfig) -> Result<SessionHandle> {
        let mut cmd = tokio::process::Command::new("claude");

        if let Some(ref prompt) = config.prompt {
            cmd.arg("--prompt").arg(prompt);
        }

        if config.yolo {
            for arg in self.yolo_args() {
                cmd.arg(arg);
            }
        }

        for arg in &config.extra_args {
            cmd.arg(arg);
        }

        let child = cmd.spawn().map_err(|e| {
            SkillxError::Agent(format!("failed to launch claude: {e}"))
        })?;

        Ok(SessionHandle {
            child: Some(child),
            lifecycle_mode: self.lifecycle_mode(),
        })
    }
}

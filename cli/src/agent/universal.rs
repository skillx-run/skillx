use async_trait::async_trait;
use std::path::PathBuf;

use super::{AgentAdapter, DetectResult, LaunchConfig, LifecycleMode, SessionHandle};
use crate::error::Result;
use crate::types::Scope;

/// Universal fallback agent using `.agents/skills/` convention.
pub struct UniversalAdapter;

#[async_trait]
impl AgentAdapter for UniversalAdapter {
    fn name(&self) -> &str {
        "universal"
    }

    fn display_name(&self) -> &str {
        "Universal Agent"
    }

    async fn detect(&self) -> DetectResult {
        // Always available as a fallback
        DetectResult {
            name: self.name().to_string(),
            detected: true,
            version: None,
            info: Some("universal fallback (.agents/skills/)".into()),
        }
    }

    fn lifecycle_mode(&self) -> LifecycleMode {
        LifecycleMode::FileInjectAndWait
    }

    fn supports_initial_prompt(&self) -> bool {
        false
    }

    fn supports_yolo(&self) -> bool {
        false
    }

    fn inject_path(&self, skill_name: &str, scope: &Scope) -> PathBuf {
        match scope {
            Scope::Project => PathBuf::from(".agents")
                .join("skills")
                .join(skill_name),
            Scope::Global => dirs::home_dir()
                .unwrap_or_default()
                .join(".agents")
                .join("skills")
                .join(skill_name),
        }
    }

    async fn launch(&self, config: LaunchConfig) -> Result<SessionHandle> {
        if let Some(ref prompt) = config.prompt {
            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                let _ = clipboard.set_text(prompt.clone());
                crate::ui::info("Prompt copied to clipboard.");
            }
        }

        crate::ui::info("Skill injected to .agents/skills/. Use your preferred agent.");
        crate::ui::info("Press Enter when done to clean up...");

        Ok(SessionHandle {
            child: None,
            lifecycle_mode: self.lifecycle_mode(),
        })
    }
}

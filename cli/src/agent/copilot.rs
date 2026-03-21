use async_trait::async_trait;
use std::path::PathBuf;

use super::{AgentAdapter, DetectResult, LaunchConfig, LifecycleMode, SessionHandle};
use crate::error::Result;
use crate::types::Scope;

pub struct CopilotAdapter;

#[async_trait]
impl AgentAdapter for CopilotAdapter {
    fn name(&self) -> &str {
        "copilot"
    }

    fn display_name(&self) -> &str {
        "GitHub Copilot"
    }

    async fn detect(&self) -> DetectResult {
        // Check for VS Code extensions directory
        let mut version = None;
        let has_extension = dirs::home_dir()
            .map(|h| {
                let ext_dir = h.join(".vscode").join("extensions");
                if ext_dir.is_dir() {
                    if let Ok(entries) = std::fs::read_dir(&ext_dir) {
                        for entry in entries.filter_map(|e| e.ok()) {
                            let name = entry.file_name().to_string_lossy().to_string();
                            if name.starts_with("github.copilot-") {
                                version = super::extract_vscode_extension_version(&name);
                                return true;
                            }
                        }
                    }
                }
                false
            })
            .unwrap_or(false);

        DetectResult {
            name: self.name().to_string(),
            detected: has_extension,
            version,
            info: if has_extension {
                Some("VS Code Copilot extension found".into())
            } else {
                None
            },
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
            Scope::Project => PathBuf::from(".github")
                .join("skills")
                .join(skill_name),
            Scope::Global => super::home_dir_or_fallback()
                .join(".github")
                .join("skills")
                .join(skill_name),
        }
    }

    async fn launch(&self, config: LaunchConfig) -> Result<SessionHandle> {
        // For IDE agents: copy prompt to clipboard and wait for user
        if let Some(ref prompt) = config.prompt {
            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                let _ = clipboard.set_text(prompt.clone());
                crate::ui::info("Prompt copied to clipboard. Paste it into Copilot Chat.");
            }
        }

        crate::ui::info("Skill injected. Open your IDE and use the skill.");
        crate::ui::info("Press Enter when done to clean up...");

        Ok(SessionHandle {
            child: None,
            lifecycle_mode: self.lifecycle_mode(),
        })
    }
}

use async_trait::async_trait;
use std::path::PathBuf;

use super::{AgentAdapter, DetectResult, LaunchConfig, LifecycleMode, SessionHandle};
use crate::error::Result;
use crate::types::Scope;

pub struct CursorAdapter;

#[async_trait]
impl AgentAdapter for CursorAdapter {
    fn name(&self) -> &str {
        "cursor"
    }

    fn display_name(&self) -> &str {
        "Cursor"
    }

    async fn detect(&self) -> DetectResult {
        let has_binary = which::which("cursor").is_ok();
        let has_process = sysinfo::System::new_all()
            .processes()
            .values()
            .any(|p| {
                let name = p.name().to_string_lossy().to_lowercase();
                name.contains("cursor")
            });

        DetectResult {
            name: self.name().to_string(),
            detected: has_binary || has_process,
            version: None,
            info: if has_binary {
                Some("cursor binary found".into())
            } else if has_process {
                Some("Cursor process detected".into())
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
            Scope::Project => PathBuf::from(".cursor")
                .join("skills")
                .join(skill_name),
            Scope::Global => dirs::home_dir()
                .unwrap_or_default()
                .join(".cursor")
                .join("skills")
                .join(skill_name),
        }
    }

    async fn launch(&self, config: LaunchConfig) -> Result<SessionHandle> {
        if let Some(ref prompt) = config.prompt {
            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                let _ = clipboard.set_text(prompt.clone());
                crate::ui::info("Prompt copied to clipboard. Paste it into Cursor chat.");
            }
        }

        crate::ui::info("Skill injected. Open Cursor and use the skill.");
        crate::ui::info("Press Enter when done to clean up...");

        Ok(SessionHandle {
            child: None,
            lifecycle_mode: self.lifecycle_mode(),
        })
    }
}

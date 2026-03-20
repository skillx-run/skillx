use async_trait::async_trait;
use std::path::PathBuf;

use super::{AgentAdapter, DetectResult, LaunchConfig, LifecycleMode, SessionHandle};
use crate::error::Result;
use crate::types::Scope;

pub struct WindsurfAdapter;

#[async_trait]
impl AgentAdapter for WindsurfAdapter {
    fn name(&self) -> &str {
        "windsurf"
    }

    fn display_name(&self) -> &str {
        "Windsurf"
    }

    async fn detect(&self) -> DetectResult {
        // Check for windsurf process
        let has_process = {
            use sysinfo::System;
            let mut sys = System::new();
            sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
            sys.processes().values().any(|p| {
                let name = p.name().to_string_lossy().to_lowercase();
                name == "windsurf"
                    || name.starts_with("windsurf ")
                    || name.starts_with("windsurf.")
            })
        };

        let has_binary = which::which("windsurf").is_ok();

        DetectResult {
            name: self.name().to_string(),
            detected: has_binary || has_process,
            version: None,
            info: if has_process {
                Some("windsurf process running".into())
            } else if has_binary {
                Some("windsurf binary found".into())
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
            Scope::Project => PathBuf::from(".windsurf")
                .join("skills")
                .join(skill_name),
            Scope::Global => super::home_dir_or_fallback()
                .join(".windsurf")
                .join("skills")
                .join(skill_name),
        }
    }

    async fn launch(&self, config: LaunchConfig) -> Result<SessionHandle> {
        if let Some(ref prompt) = config.prompt {
            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                clipboard.set_text(prompt).ok();
                eprintln!("Prompt copied to clipboard. Paste it into Windsurf chat.");
            }
        }

        eprintln!("Skill injected. Open Windsurf and use the skill.");
        eprintln!("Press Enter when done to clean up...");

        Ok(SessionHandle {
            child: None,
            lifecycle_mode: self.lifecycle_mode(),
        })
    }
}

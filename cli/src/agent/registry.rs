use super::{AgentAdapter, DetectResult};
use crate::error::{Result, SkillxError};

/// Registry of all known agent adapters.
pub struct AgentRegistry {
    adapters: Vec<Box<dyn AgentAdapter>>,
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentRegistry {
    /// Create a new registry with all built-in adapters.
    pub fn new() -> Self {
        let adapters: Vec<Box<dyn AgentAdapter>> = vec![
            // Tier 1: CLI agents
            Box::new(super::claude_code::ClaudeCodeAdapter),
            Box::new(super::codex::CodexAdapter),
            // Tier 2: CLI agents
            Box::new(super::gemini_cli::GeminiCliAdapter),
            Box::new(super::opencode::OpenCodeAdapter),
            Box::new(super::amp::AmpAdapter),
            // Tier 1: IDE agents
            Box::new(super::copilot::CopilotAdapter),
            Box::new(super::cursor::CursorAdapter),
            // Tier 2: IDE agents
            Box::new(super::windsurf::WindsurfAdapter),
            Box::new(super::cline::ClineAdapter),
            Box::new(super::roo::RooAdapter),
            // Universal fallback (always last)
            Box::new(super::universal::UniversalAdapter),
        ];
        AgentRegistry { adapters }
    }

    /// Detect all available agents.
    pub async fn detect_all(&self) -> Vec<DetectResult> {
        let mut results = Vec::new();
        for adapter in &self.adapters {
            results.push(adapter.detect().await);
        }
        results
    }

    /// Get an adapter by name.
    pub fn get(&self, name: &str) -> Option<&dyn AgentAdapter> {
        self.adapters
            .iter()
            .find(|a| a.name() == name)
            .map(|a| a.as_ref())
    }

    /// Get all adapters.
    pub fn all(&self) -> &[Box<dyn AgentAdapter>] {
        &self.adapters
    }

    /// Select an agent, with auto-detection or explicit name.
    pub async fn select(&self, name: Option<&str>) -> Result<&dyn AgentAdapter> {
        if let Some(name) = name {
            return self.get(name).ok_or_else(|| {
                SkillxError::Agent(format!("unknown agent: '{name}'"))
            });
        }

        let detected: Vec<_> = self
            .detect_all()
            .await
            .into_iter()
            .filter(|r| r.detected)
            .collect();

        match detected.len() {
            0 => Err(SkillxError::NoAgentDetected),
            1 => {
                let name = &detected[0].name;
                self.get(name).ok_or_else(|| {
                    SkillxError::Agent(format!("agent '{name}' not found in registry"))
                })
            }
            _ => {
                // Interactive selection with enhanced display
                let items: Vec<String> = detected
                    .iter()
                    .enumerate()
                    .map(|(i, d)| {
                        let adapter = self.get(&d.name).unwrap();
                        let mode = match adapter.lifecycle_mode() {
                            super::LifecycleMode::ManagedProcess => "CLI, managed-process",
                            super::LifecycleMode::FileInjectAndWait => "IDE, file-inject",
                        };
                        let recommended = if i == 0 { "  \u{2190} recommended" } else { "" };
                        format!("{}  ({mode}){recommended}", d.name)
                    })
                    .collect();
                let selection = dialoguer::Select::new()
                    .with_prompt("Multiple agents detected. Select one")
                    .items(&items)
                    .default(0)
                    .interact()
                    .map_err(|e| SkillxError::Agent(format!("selection failed: {e}")))?;

                self.get(&detected[selection].name).ok_or_else(|| {
                    SkillxError::Agent("selected agent not found".into())
                })
            }
        }
    }
}

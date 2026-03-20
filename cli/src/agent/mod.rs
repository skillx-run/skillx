pub mod claude_code;
pub mod codex;
pub mod copilot;
pub mod cursor;
pub mod registry;
pub mod universal;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::error::Result;
use crate::types::Scope;

/// How the agent lifecycle is managed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifecycleMode {
    /// skillx launches the agent as a child process and waits for exit.
    ManagedProcess,
    /// skillx injects files and waits for user to signal completion.
    FileInjectAndWait,
}

/// Result of detecting whether an agent is available.
#[derive(Debug, Clone)]
pub struct DetectResult {
    pub name: String,
    pub detected: bool,
    pub version: Option<String>,
    pub info: Option<String>,
}

/// Configuration for launching an agent.
#[derive(Debug, Clone)]
pub struct LaunchConfig {
    pub skill_name: String,
    pub skill_dir: PathBuf,
    pub prompt: Option<String>,
    pub yolo: bool,
    pub extra_args: Vec<String>,
}

/// Handle to a running agent session.
#[derive(Debug)]
pub struct SessionHandle {
    pub child: Option<tokio::process::Child>,
    pub lifecycle_mode: LifecycleMode,
}

/// Trait that each agent adapter must implement.
#[async_trait]
pub trait AgentAdapter: Send + Sync {
    /// Agent name (e.g., "claude-code").
    fn name(&self) -> &str;

    /// Display name (e.g., "Claude Code").
    fn display_name(&self) -> &str;

    /// Detect if this agent is available on the system.
    async fn detect(&self) -> DetectResult;

    /// How this agent's lifecycle is managed.
    fn lifecycle_mode(&self) -> LifecycleMode;

    /// Whether this agent supports receiving an initial prompt.
    fn supports_initial_prompt(&self) -> bool;

    /// Whether this agent supports YOLO (auto-approve) mode.
    fn supports_yolo(&self) -> bool;

    /// Arguments to pass for YOLO mode.
    fn yolo_args(&self) -> Vec<&str> {
        vec![]
    }

    /// Path where skill files should be injected.
    fn inject_path(&self, skill_name: &str, scope: &Scope) -> PathBuf;

    /// Launch the agent with the given configuration.
    async fn launch(&self, config: LaunchConfig) -> Result<SessionHandle>;

    /// Optional cleanup when session ends.
    fn on_cleanup(&self) -> Result<()> {
        Ok(())
    }
}

/// Helper: get home directory for global inject paths.
/// Falls back to a temp directory so we never silently produce an empty path.
pub fn home_dir_or_fallback() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| {
        let fallback = std::env::temp_dir().join("skillx-fallback-home");
        std::fs::create_dir_all(&fallback).ok();
        fallback
    })
}

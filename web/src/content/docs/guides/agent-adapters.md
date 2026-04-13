---
title: Writing Agent Adapters
description: How to implement a custom AgentAdapter to add support for a new AI coding agent.
---

## Overview

skillx uses the `AgentAdapter` trait to abstract differences between agents. Each adapter handles detection, injection path, launching, and cleanup for its specific agent.

If your agent isn't in the built-in registry, you can add a new adapter.

## The AgentAdapter Trait

```rust
#[async_trait]
pub trait AgentAdapter: Send + Sync {
    /// Internal name (e.g., "my-agent"). Used with --agent flag.
    fn name(&self) -> &str;

    /// Display name (e.g., "My Agent"). Shown in UI.
    fn display_name(&self) -> &str;

    /// Detect if this agent is available on the system.
    async fn detect(&self) -> DetectResult;

    /// Lifecycle mode: ManagedProcess or FileInjectAndWait.
    fn lifecycle_mode(&self) -> LifecycleMode;

    /// Whether this agent can receive an initial prompt.
    fn supports_initial_prompt(&self) -> bool;

    /// Whether this agent supports auto-approve mode.
    fn supports_auto_approve(&self) -> bool;

    /// Auto-approve mode CLI arguments (default: empty).
    fn auto_approve_args(&self) -> Vec<&str> {
        vec![]
    }

    /// Path where skill files should be injected.
    fn inject_path(&self, skill_name: &str, scope: &Scope) -> PathBuf;

    /// Launch the agent with the given configuration.
    async fn launch(&self, config: LaunchConfig) -> Result<SessionHandle>;

    /// Optional cleanup when session ends (default: no-op).
    fn on_cleanup(&self) -> Result<()> {
        Ok(())
    }
}
```

## Step-by-Step Implementation

### 1. Create the Adapter File

Create `cli/src/agent/my_agent.rs`:

```rust
use async_trait::async_trait;
use std::path::PathBuf;

use super::{AgentAdapter, DetectResult, LaunchConfig, LifecycleMode, SessionHandle};
use crate::error::{Result, SkillxError};
use crate::types::Scope;

pub struct MyAgentAdapter;

#[async_trait]
impl AgentAdapter for MyAgentAdapter {
    fn name(&self) -> &str {
        "my-agent"
    }

    fn display_name(&self) -> &str {
        "My Agent"
    }

    async fn detect(&self) -> DetectResult {
        let has_binary = which::which("my-agent").is_ok();

        DetectResult {
            name: self.name().to_string(),
            detected: has_binary,
            version: None,
            info: if has_binary {
                Some("my-agent binary found".into())
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
        true
    }

    fn auto_approve_args(&self) -> Vec<&str> {
        vec!["--auto-approve"]
    }

    fn inject_path(&self, skill_name: &str, scope: &Scope) -> PathBuf {
        match scope {
            Scope::Project => PathBuf::from(".my-agent")
                .join("skills")
                .join(skill_name),
            Scope::Global => dirs::home_dir()
                .unwrap_or_default()
                .join(".my-agent")
                .join("skills")
                .join(skill_name),
        }
    }

    async fn launch(&self, config: LaunchConfig) -> Result<SessionHandle> {
        let mut cmd = tokio::process::Command::new("my-agent");

        if let Some(ref prompt) = config.prompt {
            cmd.arg("--prompt").arg(prompt);
        }

        if config.auto_approve {
            for arg in self.auto_approve_args() {
                cmd.arg(arg);
            }
        }

        let child = cmd.spawn().map_err(|e| {
            SkillxError::Agent(format!("failed to launch my-agent: {e}"))
        })?;

        Ok(SessionHandle {
            child: Some(child),
            lifecycle_mode: self.lifecycle_mode(),
        })
    }
}
```

### 2. Register the Module

Add to `cli/src/agent/mod.rs`:

```rust
pub mod my_agent;
```

### 3. Add to the Registry

In `cli/src/agent/registry.rs`, add to `AgentRegistry::new()`:

```rust
pub fn new() -> Self {
    let adapters: Vec<Box<dyn AgentAdapter>> = vec![
        Box::new(super::claude_code::ClaudeCodeAdapter),
        Box::new(super::codex::CodexAdapter),
        Box::new(super::copilot::CopilotAdapter),
        Box::new(super::cursor::CursorAdapter),
        Box::new(super::my_agent::MyAgentAdapter),  // Add here
        Box::new(super::universal::UniversalAdapter), // Keep universal last
    ];
    AgentRegistry { adapters }
}
```

Keep Universal as the last adapter since it's the fallback.

### 4. Add Tests

In `cli/tests/unit_tests.rs`:

```rust
#[test]
fn test_my_agent_inject_path_global() {
    let adapter = MyAgentAdapter;
    let path = adapter.inject_path("test-skill", &Scope::Global);
    let home = dirs::home_dir().unwrap();
    assert_eq!(path, home.join(".my-agent/skills/test-skill"));
}

#[test]
fn test_my_agent_inject_path_project() {
    let adapter = MyAgentAdapter;
    let path = adapter.inject_path("test-skill", &Scope::Project);
    assert_eq!(path, PathBuf::from(".my-agent/skills/test-skill"));
}

#[test]
fn test_my_agent_auto_approve_args() {
    let adapter = MyAgentAdapter;
    assert!(adapter.supports_auto_approve());
    assert_eq!(adapter.auto_approve_args(), vec!["--auto-approve"]);
}
```

## Key Types

### LifecycleMode

```rust
pub enum LifecycleMode {
    ManagedProcess,     // skillx spawns and manages the agent process
    FileInjectAndWait,  // skillx injects files and waits for user input
}
```

Choose `ManagedProcess` if the agent has a CLI binary. Choose `FileInjectAndWait` if the agent runs inside an IDE.

### DetectResult

```rust
pub struct DetectResult {
    pub name: String,           // Must match adapter name()
    pub detected: bool,         // Whether the agent was found
    pub version: Option<String>, // Agent version if available
    pub info: Option<String>,    // Human-readable detection info
}
```

### LaunchConfig

```rust
pub struct LaunchConfig {
    pub skill_name: String,      // Name of the skill
    pub skill_dir: PathBuf,      // Path where skill is injected
    pub prompt: Option<String>,  // User's prompt
    pub auto_approve: bool,      // Whether auto-approve mode is requested
    pub extra_args: Vec<String>, // Additional CLI arguments
}
```

### SessionHandle

```rust
pub struct SessionHandle {
    pub child: Option<tokio::process::Child>, // None for FileInjectAndWait
    pub lifecycle_mode: LifecycleMode,
}
```

## IDE Agent Pattern

For agents that run inside an IDE:

```rust
fn lifecycle_mode(&self) -> LifecycleMode {
    LifecycleMode::FileInjectAndWait
}

async fn launch(&self, config: LaunchConfig) -> Result<SessionHandle> {
    // Copy prompt to clipboard
    if let Some(ref prompt) = config.prompt {
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            let _ = clipboard.set_text(prompt.clone());
            crate::ui::info("Prompt copied to clipboard.");
        }
    }

    crate::ui::info("Skill injected. Use your agent.");
    crate::ui::info("Press Enter when done to clean up...");

    Ok(SessionHandle {
        child: None,  // No process to manage
        lifecycle_mode: self.lifecycle_mode(),
    })
}
```

## Build and Test

```bash
cargo build
cargo test
cargo run -- agents --all  # Verify your adapter appears
```

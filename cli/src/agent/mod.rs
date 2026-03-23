pub mod amp;
pub mod claude_code;
pub mod cline;
pub mod codex;
pub mod copilot;
pub mod cursor;
pub mod gemini_cli;
pub mod generic;
pub mod opencode;
pub mod registry;
pub mod roo;
pub mod universal;
pub mod windsurf;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::LazyLock;

use std::path::Path;

use crate::error::Result;
use crate::session::inject::InjectedRecord;
use crate::types::Scope;

/// Pre-compiled regex for extracting semver-like version strings.
/// Matches `X.Y.Z` or `X.Y` preceded by a non-word/non-dot char or `v` prefix.
///
/// Known limitation: can match date-like (`2025.03.21`) or IP-like (`192.168.1`)
/// strings. This is acceptable because callers only feed it `--version` command
/// output or VS Code extension dir suffixes, where such inputs don't occur.
static VERSION_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"(?:^|[^.\w])v?(\d+\.\d+(?:\.\d+)?)\b").expect("BUG: invalid version regex")
});

/// Extract a version string from command output (e.g. `claude v1.5.2`, `codex 0.9.1`).
///
/// Looks for a semver-like pattern (`X.Y.Z` or `X.Y`) preceded by a non-word char or `v` prefix.
/// Returns `None` if no version pattern is found.
pub fn parse_version_output(output: &str) -> Option<String> {
    VERSION_RE
        .captures(output)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
}

/// Run `<binary> --version` and parse the version string.
///
/// Returns `None` on any failure (binary not found, timeout, parse error).
/// Uses a 3-second timeout to prevent blocking on unresponsive binaries.
pub async fn detect_binary_version(binary: &str) -> Option<String> {
    use std::process::Stdio;

    let child = tokio::process::Command::new(binary)
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .ok()?;

    let output = tokio::time::timeout(std::time::Duration::from_secs(3), child.wait_with_output())
        .await
        .ok()?
        .ok()?;

    let text = String::from_utf8_lossy(&output.stdout);
    // Some tools write version to stderr
    let stderr_text = String::from_utf8_lossy(&output.stderr);
    parse_version_output(&text).or_else(|| parse_version_output(&stderr_text))
}

/// Extract version from a VS Code extension directory name.
///
/// Extension dirs are named like `publisher.extension-name-1.82.0`.
/// Returns the version portion after the last `-` if it looks like a version.
pub fn extract_vscode_extension_version(dir_name: &str) -> Option<String> {
    let last_dash = dir_name.rfind('-')?;
    let version_part = &dir_name[last_dash + 1..];
    // Validate it looks like a version
    if parse_version_output(version_part).is_some() {
        Some(version_part.to_string())
    } else {
        None
    }
}

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
    pub print_mode: bool,
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

    /// Prepare skill for injection. Default: raw file copy (works for most agents).
    /// Override for agents that need aggregate file append (e.g., Goose → .goosehints).
    fn prepare_injection(
        &self,
        _skill_name: &str,
        source_dir: &Path,
        target_dir: &Path,
    ) -> Result<Vec<InjectedRecord>> {
        crate::session::inject::inject_and_collect(source_dir, target_dir)
            .map(|records| {
                records
                    .into_iter()
                    .map(|(path, sha256)| InjectedRecord::copied_file(path, sha256))
                    .collect()
            })
    }

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

#[cfg(test)]
mod version_tests {
    use super::*;

    #[test]
    fn test_parse_version_semver() {
        assert_eq!(parse_version_output("v1.5.2"), Some("1.5.2".into()));
        assert_eq!(parse_version_output("claude v1.5.2"), Some("1.5.2".into()));
        assert_eq!(parse_version_output("codex 0.9.1"), Some("0.9.1".into()));
        assert_eq!(
            parse_version_output("gemini-cli version 1.0.0"),
            Some("1.0.0".into())
        );
    }

    #[test]
    fn test_parse_version_two_part() {
        assert_eq!(parse_version_output("cursor 0.48"), Some("0.48".into()));
    }

    #[test]
    fn test_parse_version_no_match() {
        assert_eq!(parse_version_output(""), None);
        assert_eq!(parse_version_output("no version here"), None);
        assert_eq!(parse_version_output("abc"), None);
    }

    #[test]
    fn test_parse_version_stderr_format() {
        assert_eq!(
            parse_version_output("tool (version 2.3.1-beta)"),
            Some("2.3.1".into())
        );
    }

    #[test]
    fn test_parse_version_word_boundary() {
        assert_eq!(
            parse_version_output("version 1.2.3 released"),
            Some("1.2.3".into())
        );
    }

    #[test]
    fn test_parse_version_rejects_dot_prefix() {
        // Version preceded by period: [^.\w] excludes `.`
        assert_eq!(parse_version_output("lib.1.2.3"), None);
    }

    #[test]
    fn test_parse_version_rejects_underscore_prefix() {
        // Version preceded by underscore: [^.\w] excludes `_` (word char)
        assert_eq!(parse_version_output("lib_1.2.3"), None);
    }

    #[test]
    fn test_parse_version_rejects_letter_prefix() {
        // Version preceded by letter: [^.\w] excludes letters
        assert_eq!(parse_version_output("x1.2.3"), None);
    }

    #[test]
    fn test_extract_vscode_version() {
        assert_eq!(
            extract_vscode_extension_version("github.copilot-1.82.0"),
            Some("1.82.0".into())
        );
        assert_eq!(
            extract_vscode_extension_version("saoudrizwan.claude-dev-3.2.1"),
            Some("3.2.1".into())
        );
        assert_eq!(
            extract_vscode_extension_version("rooveterinaryinc.roo-cline-2.0.5"),
            Some("2.0.5".into())
        );
    }

    #[test]
    fn test_extract_vscode_version_no_version() {
        assert_eq!(extract_vscode_extension_version("some.extension"), None);
        assert_eq!(extract_vscode_extension_version("publisher.name-abc"), None);
    }
}

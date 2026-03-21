use async_trait::async_trait;
use std::path::PathBuf;

use super::{AgentAdapter, DetectResult, LaunchConfig, LifecycleMode, SessionHandle};
use crate::config::CustomAgentConfig;
use crate::error::{Result, SkillxError};
use crate::types::Scope;

/// How a generic agent is detected on the system.
#[derive(Debug, Clone)]
pub enum DetectionMethod {
    /// Check for binary via `which` + config dir existence.
    Binary,
    /// Scan `~/.vscode/extensions/` for a prefix match.
    VscodeExtension(String),
    /// Check running processes for a name match.
    Process(String),
    /// Only check if the config directory exists.
    ConfigDirOnly,
}

/// Definition for a generic agent (covers Tier 3 built-in + user custom agents).
#[derive(Debug)]
pub struct AgentDef {
    pub name: String,
    pub display_name: String,
    pub binary_name: Option<String>,
    pub config_dir: String,
    pub lifecycle: LifecycleMode,
    pub supports_prompt: bool,
    pub supports_yolo: bool,
    pub yolo_args: Vec<String>,
    pub detection: DetectionMethod,
    pub prompt_flag: Option<String>,
}

impl AgentDef {
    /// Builder for CLI agents (ManagedProcess, Binary detection).
    pub fn cli(name: &str, display: &str, binary: &str, config_dir: &str) -> Self {
        AgentDef {
            name: name.to_string(),
            display_name: display.to_string(),
            binary_name: Some(binary.to_string()),
            config_dir: config_dir.to_string(),
            lifecycle: LifecycleMode::ManagedProcess,
            supports_prompt: true,
            supports_yolo: false,
            yolo_args: Vec::new(),
            detection: DetectionMethod::Binary,
            prompt_flag: None,
        }
    }

    /// Builder for IDE agents with VS Code extension detection.
    pub fn ide_vscode(name: &str, display: &str, config_dir: &str, ext_prefix: &str) -> Self {
        AgentDef {
            name: name.to_string(),
            display_name: display.to_string(),
            binary_name: None,
            config_dir: config_dir.to_string(),
            lifecycle: LifecycleMode::FileInjectAndWait,
            supports_prompt: false,
            supports_yolo: false,
            yolo_args: Vec::new(),
            detection: DetectionMethod::VscodeExtension(ext_prefix.to_string()),
            prompt_flag: None,
        }
    }

    /// Builder for IDE agents with process detection.
    pub fn ide_process(name: &str, display: &str, config_dir: &str, proc_name: &str) -> Self {
        AgentDef {
            name: name.to_string(),
            display_name: display.to_string(),
            binary_name: None,
            config_dir: config_dir.to_string(),
            lifecycle: LifecycleMode::FileInjectAndWait,
            supports_prompt: false,
            supports_yolo: false,
            yolo_args: Vec::new(),
            detection: DetectionMethod::Process(proc_name.to_string()),
            prompt_flag: None,
        }
    }

    /// Builder for agents detected only by config dir existence.
    pub fn config_dir_only(name: &str, display: &str, config_dir: &str) -> Self {
        AgentDef {
            name: name.to_string(),
            display_name: display.to_string(),
            binary_name: None,
            config_dir: config_dir.to_string(),
            lifecycle: LifecycleMode::FileInjectAndWait,
            supports_prompt: false,
            supports_yolo: false,
            yolo_args: Vec::new(),
            detection: DetectionMethod::ConfigDirOnly,
            prompt_flag: None,
        }
    }

    /// Create from a user-defined config.toml `[[custom_agents]]` entry.
    ///
    /// Returns an error if `lifecycle` is not a recognized value.
    pub fn from_config(cfg: &CustomAgentConfig) -> std::result::Result<Self, String> {
        let lifecycle = match cfg.lifecycle.as_str() {
            "managed_process" => LifecycleMode::ManagedProcess,
            "file_inject_and_wait" => LifecycleMode::FileInjectAndWait,
            other => {
                return Err(format!(
                    "custom agent '{}': invalid lifecycle '{}' \
                     (expected 'managed_process' or 'file_inject_and_wait')",
                    cfg.name, other
                ));
            }
        };

        let detection = if cfg.binary.is_some() {
            DetectionMethod::Binary
        } else {
            DetectionMethod::ConfigDirOnly
        };

        let display_name = cfg.display_name.clone().unwrap_or_else(|| {
            // Capitalize first letter of name
            let mut chars = cfg.name.chars();
            match chars.next() {
                Some(c) => c.to_uppercase().to_string() + chars.as_str(),
                None => cfg.name.clone(),
            }
        });

        Ok(AgentDef {
            name: cfg.name.clone(),
            display_name,
            binary_name: cfg.binary.clone(),
            config_dir: cfg.config_dir.clone(),
            lifecycle,
            supports_prompt: cfg.supports_prompt,
            supports_yolo: cfg.supports_yolo,
            yolo_args: cfg.yolo_args.clone(),
            detection,
            prompt_flag: cfg.prompt_flag.clone(),
        })
    }
}

/// Generic adapter wrapping an `AgentDef`. Used for Tier 3 and custom agents.
pub struct GenericAdapter(pub AgentDef);

#[async_trait]
impl AgentAdapter for GenericAdapter {
    fn name(&self) -> &str {
        &self.0.name
    }

    fn display_name(&self) -> &str {
        &self.0.display_name
    }

    async fn detect(&self) -> DetectResult {
        let def = &self.0;
        let home = dirs::home_dir();

        let mut version = None;

        let detected = match &def.detection {
            DetectionMethod::Binary => {
                let has_binary = def
                    .binary_name
                    .as_ref()
                    .map(|b| which::which(b).is_ok())
                    .unwrap_or(false);
                let has_dir = home
                    .as_ref()
                    .map(|h| h.join(&def.config_dir).exists())
                    .unwrap_or(false);
                if has_binary {
                    // Safety: has_binary is true only when binary_name is Some
                    let bin = def.binary_name.as_deref().unwrap();
                    version = super::detect_binary_version(bin).await;
                }
                has_binary || has_dir
            }
            DetectionMethod::VscodeExtension(prefix) => home
                .as_ref()
                .map(|h| {
                    let ext_dir = h.join(".vscode").join("extensions");
                    if ext_dir.is_dir() {
                        if let Ok(entries) = std::fs::read_dir(&ext_dir) {
                            for entry in entries.flatten() {
                                let name = entry.file_name().to_string_lossy().to_string();
                                if name.starts_with(prefix.as_str()) {
                                    version = super::extract_vscode_extension_version(&name);
                                    return true;
                                }
                            }
                        }
                    }
                    false
                })
                .unwrap_or(false),
            DetectionMethod::Process(proc_name) => {
                sysinfo::System::new_all().processes().values().any(|p| {
                    let name = p.name().to_string_lossy().to_lowercase();
                    name == proc_name.to_lowercase()
                        || name.starts_with(&format!("{} ", proc_name.to_lowercase()))
                        || name.starts_with(&format!("{}.", proc_name.to_lowercase()))
                })
            }
            DetectionMethod::ConfigDirOnly => home
                .as_ref()
                .map(|h| h.join(&def.config_dir).exists())
                .unwrap_or(false),
        };

        DetectResult {
            name: def.name.clone(),
            detected,
            version,
            info: if detected {
                Some(format!("{} detected", def.display_name))
            } else {
                None
            },
        }
    }

    fn lifecycle_mode(&self) -> LifecycleMode {
        self.0.lifecycle
    }

    fn supports_initial_prompt(&self) -> bool {
        self.0.supports_prompt
    }

    fn supports_yolo(&self) -> bool {
        self.0.supports_yolo
    }

    fn yolo_args(&self) -> Vec<&str> {
        self.0.yolo_args.iter().map(|s| s.as_str()).collect()
    }

    fn inject_path(&self, skill_name: &str, scope: &Scope) -> PathBuf {
        match scope {
            Scope::Project => PathBuf::from(&self.0.config_dir)
                .join("skills")
                .join(skill_name),
            Scope::Global => super::home_dir_or_fallback()
                .join(&self.0.config_dir)
                .join("skills")
                .join(skill_name),
        }
    }

    async fn launch(&self, config: LaunchConfig) -> Result<SessionHandle> {
        let def = &self.0;

        match def.lifecycle {
            LifecycleMode::ManagedProcess => {
                let binary = def.binary_name.as_deref().ok_or_else(|| {
                    SkillxError::Agent(format!(
                        "no binary configured for managed-process agent '{}'",
                        def.name
                    ))
                })?;

                let mut cmd = tokio::process::Command::new(binary);

                if let Some(ref prompt) = config.prompt {
                    let flag = def.prompt_flag.as_deref().unwrap_or("--prompt");
                    cmd.arg(flag).arg(prompt);
                }

                if config.yolo && def.supports_yolo {
                    for arg in &def.yolo_args {
                        cmd.arg(arg);
                    }
                }

                for arg in &config.extra_args {
                    cmd.arg(arg);
                }

                let child = cmd
                    .spawn()
                    .map_err(|e| SkillxError::Agent(format!("failed to launch {binary}: {e}")))?;

                Ok(SessionHandle {
                    child: Some(child),
                    lifecycle_mode: def.lifecycle,
                })
            }
            LifecycleMode::FileInjectAndWait => {
                if let Some(ref prompt) = config.prompt {
                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                        clipboard.set_text(prompt).ok();
                        crate::ui::info(&format!(
                            "Prompt copied to clipboard. Paste it into {}.",
                            def.display_name
                        ));
                    }
                }

                crate::ui::info(&format!(
                    "Skill injected. Open {} and use the skill.",
                    def.display_name
                ));
                crate::ui::info("Press Enter when done to clean up...");

                Ok(SessionHandle {
                    child: None,
                    lifecycle_mode: def.lifecycle,
                })
            }
        }
    }
}

/// Create all 21 Tier 3 built-in agent adapters.
pub fn tier3_adapters() -> Vec<Box<dyn AgentAdapter>> {
    vec![
        // CLI agents (ManagedProcess, Binary detection)
        Box::new(GenericAdapter(AgentDef::cli(
            "goose", "Goose", "goose", ".goose",
        ))),
        Box::new(GenericAdapter(AgentDef::cli(
            "kiro", "Kiro", "kiro", ".kiro",
        ))),
        Box::new(GenericAdapter(AgentDef::cli(
            "aider", "Aider", "aider", ".aider",
        ))),
        Box::new(GenericAdapter(AgentDef::cli(
            "openclaw",
            "OpenClaw",
            "openclaw",
            ".openclaw",
        ))),
        Box::new(GenericAdapter(AgentDef::cli(
            "qwen-code",
            "Qwen Code",
            "qwen-code",
            ".qwen-code",
        ))),
        Box::new(GenericAdapter(AgentDef::cli(
            "droid", "Droid", "droid", ".droid",
        ))),
        Box::new(GenericAdapter(AgentDef::cli(
            "warp", "Warp", "warp", ".warp",
        ))),
        Box::new(GenericAdapter(AgentDef::cli(
            "openhands",
            "OpenHands",
            "openhands",
            ".openhands",
        ))),
        Box::new(GenericAdapter(AgentDef::cli(
            "command-code",
            "Command Code",
            "command-code",
            ".command-code",
        ))),
        Box::new(GenericAdapter(AgentDef::cli(
            "mistral-vibe",
            "Mistral Vibe",
            "mistral-vibe",
            ".mistral-vibe",
        ))),
        Box::new(GenericAdapter(AgentDef::cli(
            "qoder", "Qoder", "qoder", ".qoder",
        ))),
        Box::new(GenericAdapter(AgentDef::cli(
            "kode", "Kode", "kode", ".kode",
        ))),
        // IDE agents with VS Code extension detection
        Box::new(GenericAdapter(AgentDef::ide_vscode(
            "kilo",
            "Kilo Code",
            ".kilo",
            "kilocode.",
        ))),
        Box::new(GenericAdapter(AgentDef::ide_vscode(
            "augment", "Augment", ".augment", "augment.",
        ))),
        Box::new(GenericAdapter(AgentDef::ide_vscode(
            "continue",
            "Continue",
            ".continue",
            "continue.",
        ))),
        Box::new(GenericAdapter(AgentDef::ide_vscode(
            "codebuddy",
            "CodeBuddy",
            ".codebuddy",
            "codebuddy.",
        ))),
        Box::new(GenericAdapter(AgentDef::ide_vscode(
            "antigravity",
            "Antigravity",
            ".antigravity",
            "antigravity.",
        ))),
        Box::new(GenericAdapter(AgentDef::ide_vscode(
            "zencoder",
            "Zencoder",
            ".zencoder",
            "zencoder.",
        ))),
        Box::new(GenericAdapter(AgentDef::ide_vscode(
            "junie", "Junie", ".junie", "junie.",
        ))),
        // IDE agent with process detection
        Box::new(GenericAdapter(AgentDef::ide_process(
            "trae", "Trae", ".trae", "trae",
        ))),
        // ConfigDirOnly detection
        Box::new(GenericAdapter(AgentDef::config_dir_only(
            "replit-agent",
            "Replit Agent",
            ".replit",
        ))),
    ]
}

/// Create adapters from user-defined `[[custom_agents]]` in config.toml.
///
/// Logs a warning and skips any agent with an invalid lifecycle value.
pub fn custom_adapters(config: &crate::config::Config) -> Vec<Box<dyn AgentAdapter>> {
    config
        .custom_agents
        .iter()
        .filter_map(|cfg| match AgentDef::from_config(cfg) {
            Ok(def) => Some(Box::new(GenericAdapter(def)) as Box<dyn AgentAdapter>),
            Err(e) => {
                crate::ui::warn(&e);
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_agent_spec() {
        let def = AgentDef::cli("goose", "Goose", "goose", ".goose");
        assert_eq!(def.name, "goose");
        assert_eq!(def.display_name, "Goose");
        assert_eq!(def.binary_name.as_deref(), Some("goose"));
        assert_eq!(def.config_dir, ".goose");
        assert_eq!(def.lifecycle, LifecycleMode::ManagedProcess);
        assert!(def.supports_prompt);
        assert!(!def.supports_yolo);
        assert!(matches!(def.detection, DetectionMethod::Binary));
    }

    #[test]
    fn test_ide_vscode_agent_spec() {
        let def = AgentDef::ide_vscode("kilo", "Kilo Code", ".kilo", "kilocode.");
        assert_eq!(def.name, "kilo");
        assert_eq!(def.lifecycle, LifecycleMode::FileInjectAndWait);
        assert!(!def.supports_prompt);
        assert!(
            matches!(def.detection, DetectionMethod::VscodeExtension(ref p) if p == "kilocode.")
        );
    }

    #[test]
    fn test_ide_process_agent_spec() {
        let def = AgentDef::ide_process("trae", "Trae", ".trae", "trae");
        assert!(matches!(def.detection, DetectionMethod::Process(ref p) if p == "trae"));
    }

    #[test]
    fn test_inject_path_project() {
        let adapter = GenericAdapter(AgentDef::cli("goose", "Goose", "goose", ".goose"));
        let path = adapter.inject_path("my-skill", &Scope::Project);
        assert_eq!(path, PathBuf::from(".goose/skills/my-skill"));
    }

    #[test]
    fn test_inject_path_global() {
        let adapter = GenericAdapter(AgentDef::cli("goose", "Goose", "goose", ".goose"));
        let path = adapter.inject_path("my-skill", &Scope::Global);
        assert!(path.ends_with(".goose/skills/my-skill"));
        // Should be under home directory
        assert!(path.components().count() > 3);
    }

    #[test]
    fn test_tier3_names_unique() {
        let adapters = tier3_adapters();
        let names: Vec<&str> = adapters.iter().map(|a| a.name()).collect();
        let mut unique = names.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(
            names.len(),
            unique.len(),
            "duplicate Tier 3 agent names found"
        );
    }

    #[test]
    fn test_tier3_count() {
        let adapters = tier3_adapters();
        assert_eq!(adapters.len(), 21);
    }

    #[test]
    fn test_tier3_no_conflict_with_tier12() {
        let tier12_dirs = [
            ".claude",
            ".codex",
            ".copilot",
            ".cursor",
            ".gemini",
            ".opencode",
            ".amp",
            ".windsurf",
            ".cline",
            ".roo",
        ];
        let adapters = tier3_adapters();
        for adapter in &adapters {
            let generic = adapter.as_ref();
            // Check inject path uses config_dir
            let path = generic.inject_path("test", &Scope::Project);
            let config_dir = path
                .components()
                .next()
                .unwrap()
                .as_os_str()
                .to_string_lossy()
                .to_string();
            assert!(
                !tier12_dirs.contains(&config_dir.as_str()),
                "Tier 3 agent '{}' config_dir '{}' conflicts with Tier 1/2",
                generic.name(),
                config_dir
            );
        }
    }

    #[test]
    fn test_agent_def_from_config() {
        let cfg = CustomAgentConfig {
            name: "my-agent".to_string(),
            display_name: Some("My Custom Agent".to_string()),
            binary: Some("myagent".to_string()),
            config_dir: ".myagent".to_string(),
            lifecycle: "managed_process".to_string(),
            supports_prompt: true,
            supports_yolo: true,
            yolo_args: vec!["--auto".to_string()],
            prompt_flag: Some("--message".to_string()),
        };
        let def = AgentDef::from_config(&cfg).unwrap();
        assert_eq!(def.name, "my-agent");
        assert_eq!(def.display_name, "My Custom Agent");
        assert_eq!(def.binary_name.as_deref(), Some("myagent"));
        assert_eq!(def.lifecycle, LifecycleMode::ManagedProcess);
        assert!(def.supports_prompt);
        assert!(def.supports_yolo);
        assert_eq!(def.yolo_args, vec!["--auto"]);
        assert_eq!(def.prompt_flag.as_deref(), Some("--message"));

        // Wrap as GenericAdapter and verify trait methods
        let adapter = GenericAdapter(def);
        assert_eq!(adapter.name(), "my-agent");
        assert_eq!(adapter.display_name(), "My Custom Agent");
        assert!(adapter.supports_initial_prompt());
        assert!(adapter.supports_yolo());
        assert_eq!(adapter.yolo_args(), vec!["--auto"]);
    }

    #[test]
    fn test_agent_def_from_config_defaults() {
        let cfg = CustomAgentConfig {
            name: "simple".to_string(),
            display_name: None,
            binary: None,
            config_dir: ".simple".to_string(),
            lifecycle: "file_inject_and_wait".to_string(),
            supports_prompt: true,
            supports_yolo: false,
            yolo_args: vec![],
            prompt_flag: None,
        };
        let def = AgentDef::from_config(&cfg).unwrap();
        // display_name should be auto-capitalized
        assert_eq!(def.display_name, "Simple");
        assert_eq!(def.lifecycle, LifecycleMode::FileInjectAndWait);
        assert!(matches!(def.detection, DetectionMethod::ConfigDirOnly));
    }

    #[test]
    fn test_agent_def_from_config_invalid_lifecycle() {
        let cfg = CustomAgentConfig {
            name: "bad".to_string(),
            display_name: None,
            binary: None,
            config_dir: ".bad".to_string(),
            lifecycle: "invalid_value".to_string(),
            supports_prompt: true,
            supports_yolo: false,
            yolo_args: vec![],
            prompt_flag: None,
        };
        let result = AgentDef::from_config(&cfg);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("invalid lifecycle"));
        assert!(err.contains("invalid_value"));
    }
}

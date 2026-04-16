use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::error::{Result, SkillxError};

/// Global configuration loaded from `~/.skillx/config.toml`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub cache: CacheConfig,
    pub scan: ScanConfig,
    pub agent: AgentConfig,
    pub history: HistoryConfig,
    pub update: UpdateConfig,
    pub url_patterns: Vec<CustomUrlPattern>,
    pub custom_agents: Vec<CustomAgentConfig>,
}

/// User-defined URL pattern mapping a domain to a source type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomUrlPattern {
    pub domain: String,
    /// Source type string: "gitea", "gitlab", "sourcehut", "huggingface"
    pub source_type: String,
}

fn default_true() -> bool {
    true
}

/// User-defined agent configuration from config.toml `[[custom_agents]]`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomAgentConfig {
    pub name: String,
    pub display_name: Option<String>,
    pub binary: Option<String>,
    pub config_dir: String,
    /// "managed_process" or "file_inject_and_wait"
    pub lifecycle: String,
    #[serde(default = "default_true")]
    pub supports_prompt: bool,
    #[serde(default)]
    pub supports_auto_approve: bool,
    #[serde(default)]
    pub auto_approve_args: Vec<String>,
    pub prompt_flag: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CacheConfig {
    /// Cache TTL (e.g., "24h")
    pub ttl: String,
    /// Max cache size (e.g., "1GB")
    pub max_size: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ScanConfig {
    /// Default --fail-on threshold
    pub default_fail_on: String,
    /// Default headless mode (no interactive prompts)
    #[serde(default)]
    pub headless: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AgentConfig {
    pub defaults: AgentDefaults,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AgentDefaults {
    /// Preferred agent name when multiple are detected
    pub preferred: Option<String>,
    /// Default injection scope
    pub scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HistoryConfig {
    /// Max history entries to keep
    pub max_entries: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UpdateConfig {
    /// Whether to check for CLI updates. Default: true
    pub check: bool,
    /// Check interval (e.g., "24h", "7d"). Default: "24h"
    pub interval: String,
}

impl Default for UpdateConfig {
    fn default() -> Self {
        UpdateConfig {
            check: true,
            interval: "24h".to_string(),
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        CacheConfig {
            ttl: "24h".to_string(),
            max_size: "1GB".to_string(),
        }
    }
}

impl Default for ScanConfig {
    fn default() -> Self {
        ScanConfig {
            default_fail_on: "danger".to_string(),
            headless: false,
        }
    }
}

impl Default for AgentDefaults {
    fn default() -> Self {
        AgentDefaults {
            preferred: None,
            scope: "global".to_string(),
        }
    }
}

impl Default for HistoryConfig {
    fn default() -> Self {
        HistoryConfig { max_entries: 50 }
    }
}

impl Config {
    /// Load config from `~/.skillx/config.toml`, or return defaults if not found.
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        if path.exists() {
            let content = std::fs::read_to_string(&path)
                .map_err(|e| SkillxError::Config(format!("failed to read config: {e}")))?;
            let config: Config = toml::from_str(&content)
                .map_err(|e| SkillxError::Config(format!("failed to parse config: {e}")))?;
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }

    /// Ensure all required directories exist under `~/.skillx/`.
    pub fn ensure_dirs() -> Result<()> {
        let base = Self::base_dir()?;
        let dirs = [
            base.clone(),
            base.join("cache"),
            base.join("active"),
            base.join("history"),
        ];
        for dir in &dirs {
            std::fs::create_dir_all(dir).map_err(|e| {
                SkillxError::Config(format!("failed to create {}: {e}", dir.display()))
            })?;
        }
        Ok(())
    }

    /// Base directory: `$SKILLX_HOME` if set, otherwise `~/.skillx/`.
    pub fn base_dir() -> Result<PathBuf> {
        if let Ok(home) = std::env::var("SKILLX_HOME") {
            return Ok(PathBuf::from(home));
        }
        Ok(dirs::home_dir()
            .ok_or_else(|| SkillxError::Config("could not determine home directory".into()))?
            .join(".skillx"))
    }

    /// Config file path: `~/.skillx/config.toml`
    pub fn config_path() -> Result<PathBuf> {
        Ok(Self::base_dir()?.join("config.toml"))
    }

    /// Cache directory: `~/.skillx/cache/`
    pub fn cache_dir() -> Result<PathBuf> {
        Ok(Self::base_dir()?.join("cache"))
    }

    /// Active sessions directory: `~/.skillx/active/`
    pub fn active_dir() -> Result<PathBuf> {
        Ok(Self::base_dir()?.join("active"))
    }

    /// History directory: `~/.skillx/history/`
    pub fn history_dir() -> Result<PathBuf> {
        Ok(Self::base_dir()?.join("history"))
    }

    /// Parse TTL string (e.g., "24h", "7d") into seconds.
    pub fn ttl_seconds(&self) -> u64 {
        parse_duration_secs(&self.cache.ttl).unwrap_or(86400)
    }

    /// Parse update check interval into seconds.
    pub fn update_check_interval_secs(&self) -> u64 {
        parse_duration_secs(&self.update.interval).unwrap_or(86400)
    }
}

/// Parse a human-friendly duration string into seconds.
pub fn parse_duration_secs(s: &str) -> Option<u64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    let (num_str, suffix) = if s.ends_with('s') && !s.ends_with("ms") {
        (s.strip_suffix('s').unwrap_or(s), "s")
    } else if s.ends_with('m') && !s.ends_with("ms") {
        (s.strip_suffix('m').unwrap_or(s), "m")
    } else if let Some(stripped) = s.strip_suffix('h') {
        (stripped, "h")
    } else if let Some(stripped) = s.strip_suffix('d') {
        (stripped, "d")
    } else {
        (s, "s")
    };

    let num: u64 = num_str.parse().ok()?;
    let multiplier = match suffix {
        "s" => 1,
        "m" => 60,
        "h" => 3600,
        "d" => 86400,
        _ => return None,
    };

    Some(num * multiplier)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_url_patterns() {
        let toml_str = r#"
[[url_patterns]]
domain = "mygitea.company.com"
source_type = "gitea"

[[url_patterns]]
domain = "mylab.example.com"
source_type = "gitlab"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.url_patterns.len(), 2);
        assert_eq!(config.url_patterns[0].domain, "mygitea.company.com");
        assert_eq!(config.url_patterns[0].source_type, "gitea");
        assert_eq!(config.url_patterns[1].source_type, "gitlab");
    }

    #[test]
    fn test_parse_custom_agents() {
        let toml_str = r#"
[[custom_agents]]
name = "my-cli-agent"
display_name = "My CLI Agent"
binary = "mycli"
config_dir = ".mycli"
lifecycle = "managed_process"
supports_prompt = true
supports_auto_approve = true
auto_approve_args = ["--yes"]
prompt_flag = "--message"

[[custom_agents]]
name = "my-ide-agent"
config_dir = ".myide"
lifecycle = "file_inject_and_wait"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.custom_agents.len(), 2);

        let cli = &config.custom_agents[0];
        assert_eq!(cli.name, "my-cli-agent");
        assert_eq!(cli.display_name.as_deref(), Some("My CLI Agent"));
        assert_eq!(cli.binary.as_deref(), Some("mycli"));
        assert_eq!(cli.lifecycle, "managed_process");
        assert!(cli.supports_prompt);
        assert!(cli.supports_auto_approve);
        assert_eq!(cli.auto_approve_args, vec!["--yes"]);
        assert_eq!(cli.prompt_flag.as_deref(), Some("--message"));

        let ide = &config.custom_agents[1];
        assert_eq!(ide.name, "my-ide-agent");
        assert!(ide.display_name.is_none());
        assert!(ide.binary.is_none());
        assert_eq!(ide.lifecycle, "file_inject_and_wait");
        // default: supports_prompt = true
        assert!(ide.supports_prompt);
        // default: supports_auto_approve = false
        assert!(!ide.supports_auto_approve);
    }

    #[test]
    fn test_empty_url_patterns_and_custom_agents_default() {
        let config: Config = toml::from_str("").unwrap();
        assert!(config.url_patterns.is_empty());
        assert!(config.custom_agents.is_empty());
    }

    #[test]
    fn test_update_config_defaults() {
        let config: Config = toml::from_str("").unwrap();
        assert!(config.update.check);
        assert_eq!(config.update.interval, "24h");
    }

    #[test]
    fn test_update_config_custom() {
        let toml_str = r#"
[update]
check = false
interval = "7d"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert!(!config.update.check);
        assert_eq!(config.update.interval, "7d");
    }

    #[test]
    fn test_update_check_interval_secs() {
        let mut config = Config::default();
        assert_eq!(config.update_check_interval_secs(), 86400); // 24h
        config.update.interval = "7d".to_string();
        assert_eq!(config.update_check_interval_secs(), 604800); // 7d
        config.update.interval = "1h".to_string();
        assert_eq!(config.update_check_interval_secs(), 3600); // 1h
    }

    #[test]
    fn test_full_config_with_all_sections() {
        let toml_str = r#"
[cache]
ttl = "48h"

[scan]
default_fail_on = "warn"

[agent.defaults]
preferred = "claude-code"
scope = "project"

[history]
max_entries = 100

[update]
check = false
interval = "12h"

[[url_patterns]]
domain = "git.example.com"
source_type = "gitea"

[[custom_agents]]
name = "test-agent"
config_dir = ".test"
lifecycle = "managed_process"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.cache.ttl, "48h");
        assert_eq!(config.scan.default_fail_on, "warn");
        assert_eq!(
            config.agent.defaults.preferred.as_deref(),
            Some("claude-code")
        );
        assert_eq!(config.history.max_entries, 100);
        assert!(!config.update.check);
        assert_eq!(config.update.interval, "12h");
        assert_eq!(config.url_patterns.len(), 1);
        assert_eq!(config.custom_agents.len(), 1);
    }
}

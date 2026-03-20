use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::error::{Result, SkillxError};

/// Global configuration loaded from `~/.skillx/config.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub cache: CacheConfig,
    pub scan: ScanConfig,
    pub agent: AgentConfig,
    pub history: HistoryConfig,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl Default for Config {
    fn default() -> Self {
        Config {
            cache: CacheConfig::default(),
            scan: ScanConfig::default(),
            agent: AgentConfig::default(),
            history: HistoryConfig::default(),
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
        }
    }
}

impl Default for AgentConfig {
    fn default() -> Self {
        AgentConfig {
            defaults: AgentDefaults::default(),
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
        let path = Self::config_path();
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
        let base = Self::base_dir();
        let dirs = [
            base.clone(),
            base.join("cache"),
            base.join("active"),
            base.join("history"),
        ];
        for dir in &dirs {
            std::fs::create_dir_all(dir)
                .map_err(|e| SkillxError::Config(format!("failed to create {}: {e}", dir.display())))?;
        }
        Ok(())
    }

    /// Base directory: `~/.skillx/`
    pub fn base_dir() -> PathBuf {
        dirs::home_dir()
            .expect("could not determine home directory")
            .join(".skillx")
    }

    /// Config file path: `~/.skillx/config.toml`
    pub fn config_path() -> PathBuf {
        Self::base_dir().join("config.toml")
    }

    /// Cache directory: `~/.skillx/cache/`
    pub fn cache_dir() -> PathBuf {
        Self::base_dir().join("cache")
    }

    /// Active sessions directory: `~/.skillx/active/`
    pub fn active_dir() -> PathBuf {
        Self::base_dir().join("active")
    }

    /// History directory: `~/.skillx/history/`
    pub fn history_dir() -> PathBuf {
        Self::base_dir().join("history")
    }

    /// Parse TTL string (e.g., "24h", "7d") into seconds.
    pub fn ttl_seconds(&self) -> u64 {
        parse_duration_secs(&self.cache.ttl).unwrap_or(86400)
    }
}

/// Parse a human-friendly duration string into seconds.
pub fn parse_duration_secs(s: &str) -> Option<u64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    let (num_str, suffix) = if s.ends_with('s') && !s.ends_with("ms") {
        (&s[..s.len() - 1], "s")
    } else if s.ends_with('m') && !s.ends_with("ms") {
        (&s[..s.len() - 1], "m")
    } else if s.ends_with('h') {
        (&s[..s.len() - 1], "h")
    } else if s.ends_with('d') {
        (&s[..s.len() - 1], "d")
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

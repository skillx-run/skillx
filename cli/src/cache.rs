use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::error::{Result, SkillxError};

/// Metadata stored alongside cached skill files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMeta {
    pub source: String,
    pub cached_at: DateTime<Utc>,
    pub ttl_seconds: u64,
    pub skill_name: Option<String>,
}

pub struct CacheManager;

impl CacheManager {
    /// Compute a cache key (SHA256 hash) from a source string.
    pub fn source_hash(source: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(source.as_bytes());
        format!("{:x}", hasher.finalize())[..16].to_string()
    }

    /// Look up a cached skill. Returns the cache directory if valid (not expired).
    pub fn lookup(source: &str) -> Result<Option<PathBuf>> {
        let hash = Self::source_hash(source);
        let cache_dir = Config::cache_dir().join(&hash);
        let meta_path = cache_dir.join("meta.json");

        if !meta_path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&meta_path)
            .map_err(|e| SkillxError::Cache(format!("failed to read cache meta: {e}")))?;
        let meta: CacheMeta = serde_json::from_str(&content)
            .map_err(|e| SkillxError::Cache(format!("failed to parse cache meta: {e}")))?;

        // Check TTL
        let config = Config::load().unwrap_or_default();
        let ttl = config.ttl_seconds();
        let age = Utc::now()
            .signed_duration_since(meta.cached_at)
            .num_seconds();

        if age < 0 || age as u64 > ttl {
            return Ok(None); // Expired
        }

        let skill_dir = cache_dir.join("skill-files");
        if skill_dir.is_dir() {
            Ok(Some(skill_dir))
        } else {
            Ok(None)
        }
    }

    /// Store skill files in cache.
    pub fn store(source: &str, source_dir: &Path, skill_name: Option<&str>) -> Result<PathBuf> {
        let hash = Self::source_hash(source);
        let cache_dir = Config::cache_dir().join(&hash);
        let skill_dir = cache_dir.join("skill-files");

        // Remove old cache entry
        if cache_dir.exists() {
            std::fs::remove_dir_all(&cache_dir).ok();
        }

        // Copy files
        std::fs::create_dir_all(&skill_dir).map_err(|e| {
            SkillxError::Cache(format!("failed to create cache dir: {e}"))
        })?;
        copy_dir_all(source_dir, &skill_dir)?;

        // Write meta
        let config = Config::load().unwrap_or_default();
        let meta = CacheMeta {
            source: source.to_string(),
            cached_at: Utc::now(),
            ttl_seconds: config.ttl_seconds(),
            skill_name: skill_name.map(|s| s.to_string()),
        };
        let meta_json = serde_json::to_string_pretty(&meta)
            .map_err(|e| SkillxError::Cache(format!("failed to serialize cache meta: {e}")))?;
        std::fs::write(cache_dir.join("meta.json"), meta_json)
            .map_err(|e| SkillxError::Cache(format!("failed to write cache meta: {e}")))?;

        Ok(skill_dir)
    }

    /// List all cached skills.
    pub fn list() -> Result<Vec<CacheMeta>> {
        let cache_dir = Config::cache_dir();
        if !cache_dir.exists() {
            return Ok(vec![]);
        }

        let mut entries = Vec::new();
        let dir_entries = std::fs::read_dir(&cache_dir)
            .map_err(|e| SkillxError::Cache(format!("failed to read cache dir: {e}")))?;

        for entry in dir_entries {
            let entry = entry
                .map_err(|e| SkillxError::Cache(format!("dir entry error: {e}")))?;
            let meta_path = entry.path().join("meta.json");
            if meta_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&meta_path) {
                    if let Ok(meta) = serde_json::from_str::<CacheMeta>(&content) {
                        entries.push(meta);
                    }
                }
            }
        }

        Ok(entries)
    }

    /// Clean all cache entries.
    pub fn clean() -> Result<usize> {
        let cache_dir = Config::cache_dir();
        if !cache_dir.exists() {
            return Ok(0);
        }

        let mut count = 0;
        let entries = std::fs::read_dir(&cache_dir)
            .map_err(|e| SkillxError::Cache(format!("failed to read cache dir: {e}")))?;

        for entry in entries {
            let entry = entry
                .map_err(|e| SkillxError::Cache(format!("dir entry error: {e}")))?;
            if entry.path().is_dir() {
                std::fs::remove_dir_all(entry.path()).ok();
                count += 1;
            }
        }

        Ok(count)
    }
}

/// Recursively copy all files from src to dst.
fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)
        .map_err(|e| SkillxError::Cache(format!("failed to create dir: {e}")))?;

    let entries = std::fs::read_dir(src)
        .map_err(|e| SkillxError::Cache(format!("failed to read dir: {e}")))?;

    for entry in entries {
        let entry = entry.map_err(|e| SkillxError::Cache(format!("dir entry error: {e}")))?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path).map_err(|e| {
                SkillxError::Cache(format!(
                    "failed to copy {} to {}: {e}",
                    src_path.display(),
                    dst_path.display()
                ))
            })?;
        }
    }

    Ok(())
}

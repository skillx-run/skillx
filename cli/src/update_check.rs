use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::{DateTime, Utc};
use console::style;
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::error::{Result, SkillxError};

/// Cached result of the last version check, stored at `~/.skillx/update-check.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCheckCache {
    pub last_checked: DateTime<Utc>,
    pub latest_version: String,
    pub current_version: String,
}

/// Result of a version check — a newer version is available.
#[derive(Debug, Clone)]
pub struct UpdateAvailable {
    pub current: String,
    pub latest: String,
    pub install_method: InstallMethod,
}

/// How skillx was installed, determines upgrade behavior.
#[derive(Debug, Clone, PartialEq)]
pub enum InstallMethod {
    Homebrew,
    Cargo,
    CargoBinstall,
    Unknown,
}

/// Path to the cache file: `~/.skillx/update-check.json`.
pub fn cache_path() -> Result<PathBuf> {
    Ok(Config::base_dir()?.join("update-check.json"))
}

/// Load the cached check result. Returns None if file doesn't exist or is malformed.
pub fn load_cache() -> Option<UpdateCheckCache> {
    let path = cache_path().ok()?;
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

/// Save cache to disk. Silently ignores errors.
pub fn save_cache(cache: &UpdateCheckCache) {
    if let Ok(path) = cache_path() {
        let _ = serde_json::to_string(cache)
            .ok()
            .and_then(|json| std::fs::write(path, json).ok());
    }
}

/// Check if we need to make an API call to check for updates.
///
/// Returns false if:
/// - `SKILLX_NO_UPDATE_CHECK` env var is set
/// - `config.update.check` is false
/// - Check interval hasn't elapsed and binary version hasn't changed
pub fn should_check(config: &Config) -> bool {
    if std::env::var("SKILLX_NO_UPDATE_CHECK").is_ok() {
        return false;
    }
    if !config.update.check {
        return false;
    }
    match load_cache() {
        Some(cache) => {
            let current = env!("CARGO_PKG_VERSION");
            // Re-check if the binary version changed (user upgraded)
            if cache.current_version != current {
                return true;
            }
            let elapsed = Utc::now()
                .signed_duration_since(cache.last_checked)
                .num_seconds();
            // Also re-check if clock went backwards
            elapsed < 0 || elapsed as u64 >= config.update_check_interval_secs()
        }
        None => true, // never checked before
    }
}

/// Check if a cached result indicates an available update, without making an API call.
///
/// Used for showing notifications on every run even when the API check interval hasn't elapsed.
pub fn cached_update_available() -> Option<UpdateAvailable> {
    let cache = load_cache()?;
    let current = env!("CARGO_PKG_VERSION");
    if is_newer(current, &cache.latest_version) {
        Some(UpdateAvailable {
            current: current.to_string(),
            latest: cache.latest_version,
            install_method: detect_install_method(),
        })
    } else {
        None
    }
}

/// Fetch the latest version, trying GitHub Releases API first, then crates.io as fallback.
///
/// Returns the version string without the `v` prefix.
/// `timeout` controls the HTTP client timeout (3s for background, 10s for explicit command).
pub async fn fetch_latest_version(timeout: Duration) -> Result<String> {
    let client = reqwest::Client::builder()
        .user_agent(format!("skillx/{}", env!("CARGO_PKG_VERSION")))
        .timeout(timeout)
        .build()
        .map_err(|e| SkillxError::Network(format!("failed to build HTTP client: {e}")))?;

    // Try GitHub Releases API first (most authoritative — tracks release tags)
    match fetch_from_github(&client).await {
        Ok(version) => return Ok(version),
        Err(_) => {
            // Fall back to crates.io (independent infrastructure, no shared rate limit)
        }
    }

    fetch_from_crates_io(&client).await
}

/// Fetch latest version from GitHub Releases API.
async fn fetch_from_github(client: &reqwest::Client) -> Result<String> {
    let mut req = client
        .get("https://api.github.com/repos/skillx-run/skillx/releases/latest")
        .header("Accept", "application/vnd.github+json");

    // Use GITHUB_TOKEN if available for higher rate limits (5000/hr vs 60/hr)
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        req = req.header("Authorization", format!("Bearer {token}"));
    }

    let resp = req
        .send()
        .await
        .map_err(|e| SkillxError::Network(format!("GitHub API request failed: {e}")))?;

    if !resp.status().is_success() {
        return Err(SkillxError::Network(format!(
            "GitHub API returned {}",
            resp.status()
        )));
    }

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| SkillxError::Network(format!("failed to parse GitHub response: {e}")))?;

    let tag = body["tag_name"]
        .as_str()
        .ok_or_else(|| SkillxError::Network("missing tag_name in GitHub response".into()))?;

    Ok(tag.strip_prefix('v').unwrap_or(tag).to_string())
}

/// Fetch latest version from crates.io API (fallback).
async fn fetch_from_crates_io(client: &reqwest::Client) -> Result<String> {
    let resp = client
        .get("https://crates.io/api/v1/crates/skillx")
        .send()
        .await
        .map_err(|e| SkillxError::Network(format!("crates.io request failed: {e}")))?;

    if !resp.status().is_success() {
        return Err(SkillxError::Network(format!(
            "crates.io returned {}",
            resp.status()
        )));
    }

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| SkillxError::Network(format!("failed to parse crates.io response: {e}")))?;

    body["crate"]["max_version"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| {
            SkillxError::Network("missing max_version in crates.io response".into())
        })
}

/// Compare two semver version strings. Returns true if `latest` > `current`.
pub fn is_newer(current: &str, latest: &str) -> bool {
    match (
        semver::Version::parse(current),
        semver::Version::parse(latest),
    ) {
        (Ok(c), Ok(l)) => l > c,
        _ => false, // if either fails to parse, assume not newer (safe default)
    }
}

/// Detect how skillx was installed by examining the current executable path.
pub fn detect_install_method() -> InstallMethod {
    std::env::current_exe()
        .ok()
        .map(|p| detect_install_method_from_path(&p))
        .unwrap_or(InstallMethod::Unknown)
}

/// Testable version of install method detection from a given path.
pub fn detect_install_method_from_path(path: &Path) -> InstallMethod {
    let path_str = path.to_string_lossy();

    // Homebrew: /opt/homebrew/Cellar/..., /usr/local/Cellar/..., /home/linuxbrew/...
    if path_str.contains("/Cellar/")
        || path_str.contains("/homebrew/")
        || path_str.contains("/linuxbrew/")
    {
        return InstallMethod::Homebrew;
    }

    // Cargo: ~/.cargo/bin/skillx
    if path_str.contains("/.cargo/bin/") {
        // Check if cargo-binstall is available (prefer it for faster upgrades)
        if which::which("cargo-binstall").is_ok() {
            return InstallMethod::CargoBinstall;
        }
        return InstallMethod::Cargo;
    }

    InstallMethod::Unknown
}

/// Perform the full background check: fetch, compare, cache, return update info if available.
pub async fn check_for_update() -> Option<UpdateAvailable> {
    let current = env!("CARGO_PKG_VERSION");
    let latest = fetch_latest_version(Duration::from_secs(3)).await.ok()?;

    // Save cache regardless of whether update is available
    save_cache(&UpdateCheckCache {
        last_checked: Utc::now(),
        latest_version: latest.clone(),
        current_version: current.to_string(),
    });

    if is_newer(current, &latest) {
        Some(UpdateAvailable {
            current: current.to_string(),
            latest,
            install_method: detect_install_method(),
        })
    } else {
        None
    }
}

/// Format a background notification message for the user.
pub fn format_update_message(update: &UpdateAvailable) -> String {
    format!(
        "{} A new version of skillx is available: {} {} {}\n  Run {} to update.",
        style("ℹ").blue().bold(),
        style(format!("v{}", update.current)).red(),
        style("→").dim(),
        style(format!("v{}", update.latest)).green().bold(),
        style("`skillx upgrade`").bold(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write as _;
    use std::sync::Mutex;

    /// Mutex to serialize tests that modify environment variables.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    /// Helper: set up an isolated SKILLX_HOME in a temp dir.
    /// Returns the lock guard — hold it for the duration of the test.
    fn setup_env(tmp: &tempfile::TempDir) -> std::sync::MutexGuard<'static, ()> {
        let guard = ENV_LOCK.lock().unwrap();
        std::env::set_var("SKILLX_HOME", tmp.path());
        std::env::remove_var("SKILLX_NO_UPDATE_CHECK");
        guard
    }

    /// Helper: write an UpdateCheckCache to a specific path.
    fn write_cache_to(dir: &Path, cache: &UpdateCheckCache) {
        let path = dir.join("update-check.json");
        let json = serde_json::to_string(cache).unwrap();
        let mut f = std::fs::File::create(path).unwrap();
        f.write_all(json.as_bytes()).unwrap();
    }

    #[test]
    fn test_should_check_env_var_disables() {
        let tmp = tempfile::tempdir().unwrap();
        let _guard = setup_env(&tmp);
        std::env::set_var("SKILLX_NO_UPDATE_CHECK", "1");

        let config = Config::default();
        assert!(!should_check(&config));
    }

    #[test]
    fn test_should_check_config_disables() {
        let tmp = tempfile::tempdir().unwrap();
        let _guard = setup_env(&tmp);

        let mut config = Config::default();
        config.update.check = false;
        assert!(!should_check(&config));
    }

    #[test]
    fn test_should_check_no_cache_returns_true() {
        let tmp = tempfile::tempdir().unwrap();
        let _guard = setup_env(&tmp);

        let config = Config::default();
        assert!(should_check(&config));
    }

    #[test]
    fn test_should_check_interval_not_elapsed() {
        let tmp = tempfile::tempdir().unwrap();
        let _guard = setup_env(&tmp);

        write_cache_to(tmp.path(), &UpdateCheckCache {
            last_checked: Utc::now(),
            latest_version: "0.6.0".to_string(),
            current_version: env!("CARGO_PKG_VERSION").to_string(),
        });

        let config = Config::default(); // 24h interval
        assert!(!should_check(&config));
    }

    #[test]
    fn test_should_check_interval_elapsed() {
        let tmp = tempfile::tempdir().unwrap();
        let _guard = setup_env(&tmp);

        write_cache_to(tmp.path(), &UpdateCheckCache {
            last_checked: Utc::now() - chrono::Duration::hours(25),
            latest_version: "0.6.0".to_string(),
            current_version: env!("CARGO_PKG_VERSION").to_string(),
        });

        let config = Config::default(); // 24h interval
        assert!(should_check(&config));
    }

    #[test]
    fn test_should_check_version_changed_forces_recheck() {
        let tmp = tempfile::tempdir().unwrap();
        let _guard = setup_env(&tmp);

        write_cache_to(tmp.path(), &UpdateCheckCache {
            last_checked: Utc::now(),
            latest_version: "0.6.0".to_string(),
            current_version: "0.0.1".to_string(), // different from actual
        });

        let config = Config::default();
        assert!(should_check(&config));
    }

    #[test]
    fn test_cached_update_available_with_newer() {
        let tmp = tempfile::tempdir().unwrap();
        let _guard = setup_env(&tmp);

        write_cache_to(tmp.path(), &UpdateCheckCache {
            last_checked: Utc::now(),
            latest_version: "99.0.0".to_string(), // definitely newer
            current_version: env!("CARGO_PKG_VERSION").to_string(),
        });

        let result = cached_update_available();
        assert!(result.is_some());
        let update = result.unwrap();
        assert_eq!(update.latest, "99.0.0");
    }

    #[test]
    fn test_cached_update_available_already_latest() {
        let tmp = tempfile::tempdir().unwrap();
        let _guard = setup_env(&tmp);

        let current = env!("CARGO_PKG_VERSION");
        write_cache_to(tmp.path(), &UpdateCheckCache {
            last_checked: Utc::now(),
            latest_version: current.to_string(),
            current_version: current.to_string(),
        });

        assert!(cached_update_available().is_none());
    }

    #[test]
    fn test_cached_update_available_no_cache() {
        let tmp = tempfile::tempdir().unwrap();
        let _guard = setup_env(&tmp);

        assert!(cached_update_available().is_none());
    }

    #[test]
    fn test_save_and_load_cache_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let _guard = setup_env(&tmp);

        let cache = UpdateCheckCache {
            last_checked: Utc::now(),
            latest_version: "1.2.3".to_string(),
            current_version: "0.6.0".to_string(),
        };
        save_cache(&cache);

        let loaded = load_cache().unwrap();
        assert_eq!(loaded.latest_version, "1.2.3");
        assert_eq!(loaded.current_version, "0.6.0");
    }

    #[test]
    fn test_is_newer_basic() {
        assert!(is_newer("0.6.0", "0.7.0"));
        assert!(is_newer("0.6.0", "1.0.0"));
        assert!(is_newer("0.1.0", "0.1.1"));
    }

    #[test]
    fn test_is_newer_not_newer() {
        assert!(!is_newer("0.7.0", "0.6.0"));
        assert!(!is_newer("1.0.0", "0.9.0"));
    }

    #[test]
    fn test_is_newer_equal() {
        assert!(!is_newer("0.6.0", "0.6.0"));
        assert!(!is_newer("1.0.0", "1.0.0"));
    }

    #[test]
    fn test_is_newer_prerelease() {
        // 0.7.0-rc.1 > 0.6.0 per semver
        assert!(is_newer("0.6.0", "0.7.0-rc.1"));
        // 0.7.0-rc.1 < 0.7.0 per semver
        assert!(!is_newer("0.7.0", "0.7.0-rc.1"));
    }

    #[test]
    fn test_is_newer_invalid_version() {
        assert!(!is_newer("not-a-version", "0.7.0"));
        assert!(!is_newer("0.6.0", "not-a-version"));
        assert!(!is_newer("abc", "def"));
    }

    #[test]
    fn test_is_newer_multidigit() {
        assert!(is_newer("0.9.0", "0.10.0"));
        assert!(is_newer("0.99.0", "0.100.0"));
    }

    #[test]
    fn test_cache_roundtrip() {
        let cache = UpdateCheckCache {
            last_checked: Utc::now(),
            latest_version: "0.7.0".to_string(),
            current_version: "0.6.0".to_string(),
        };
        let json = serde_json::to_string(&cache).unwrap();
        let loaded: UpdateCheckCache = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.latest_version, "0.7.0");
        assert_eq!(loaded.current_version, "0.6.0");
    }

    #[test]
    fn test_detect_homebrew_macos() {
        assert_eq!(
            detect_install_method_from_path(Path::new(
                "/opt/homebrew/Cellar/skillx/0.6.0/bin/skillx"
            )),
            InstallMethod::Homebrew
        );
    }

    #[test]
    fn test_detect_homebrew_linux() {
        assert_eq!(
            detect_install_method_from_path(Path::new(
                "/home/linuxbrew/.linuxbrew/Cellar/skillx/0.6.0/bin/skillx"
            )),
            InstallMethod::Homebrew
        );
    }

    #[test]
    fn test_detect_homebrew_usr_local() {
        assert_eq!(
            detect_install_method_from_path(Path::new(
                "/usr/local/Cellar/skillx/0.6.0/bin/skillx"
            )),
            InstallMethod::Homebrew
        );
    }

    #[test]
    fn test_detect_cargo() {
        // Note: this test may detect CargoBinstall if cargo-binstall is installed on the test machine.
        // We test the path matching, not the which::which fallback.
        let method =
            detect_install_method_from_path(Path::new("/home/user/.cargo/bin/skillx"));
        assert!(
            method == InstallMethod::Cargo || method == InstallMethod::CargoBinstall
        );
    }

    #[test]
    fn test_detect_unknown() {
        assert_eq!(
            detect_install_method_from_path(Path::new("/usr/local/bin/skillx")),
            InstallMethod::Unknown
        );
        assert_eq!(
            detect_install_method_from_path(Path::new("/tmp/skillx")),
            InstallMethod::Unknown
        );
    }

    #[test]
    fn test_format_update_message_contains_versions() {
        let update = UpdateAvailable {
            current: "0.6.0".to_string(),
            latest: "0.7.0".to_string(),
            install_method: InstallMethod::Homebrew,
        };
        let msg = format_update_message(&update);
        assert!(msg.contains("0.6.0"));
        assert!(msg.contains("0.7.0"));
        assert!(msg.contains("skillx upgrade"));
    }
}

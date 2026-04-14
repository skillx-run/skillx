//! Shared utilities for git-clone-based fetching and archive tarball downloads.
//!
//! All git-based platforms (GitHub, GitLab, Bitbucket, Gitea, SourceHut) share
//! a common three-tier download strategy:
//!   1. Archive tarball (no API, no git needed)
//!   2. Git shallow clone (HTTPS first, SSH fallback)
//!   3. Platform API with retry (last resort)

use std::path::{Path, PathBuf};
use std::process::Output;

use crate::config::Config;
use crate::error::{Result, SkillxError};
use crate::ui;

// ---------------------------------------------------------------------------
// Git clone
// ---------------------------------------------------------------------------

/// Fetch a skill via `git clone`.
///
/// Tries HTTPS first, then SSH (with a 3-second probe). Returns `Some(files)`
/// on success, `None` when git is unavailable or all attempts fail (caller
/// should fall back to the platform API).
pub async fn clone_skill(
    https_url: &str,
    ssh_url: Option<&str>,
    subpath: Option<&str>,
    ref_: Option<&str>,
    dest: &Path,
) -> Option<Vec<PathBuf>> {
    if !is_git_available() {
        return None;
    }

    // Try HTTPS clone first
    if let Some(files) = try_clone(https_url, subpath, ref_, dest).await {
        return Some(files);
    }

    // HTTPS failed — try SSH if a URL was provided
    if let Some(ssh) = ssh_url {
        // Extract host from SSH URL for probe (e.g. "git@github.com:o/r.git" → "github.com")
        if let Some(host) = extract_ssh_host(ssh) {
            if ssh_probe(&host).await {
                if let Some(files) = try_clone(ssh, subpath, ref_, dest).await {
                    return Some(files);
                }
            }
        }
    }

    None
}

/// Attempt a single git clone + optional sparse checkout.
async fn try_clone(
    url: &str,
    subpath: Option<&str>,
    ref_: Option<&str>,
    dest: &Path,
) -> Option<Vec<PathBuf>> {
    let tmp_dir = Config::cache_dir()
        .ok()?
        .join(format!("tmp-clone-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&tmp_dir).ok()?;
    let clone_dir = tmp_dir.join("repo");

    // Build clone args
    let mut args: Vec<&str> = vec!["clone", "--depth", "1"];

    // --branch for non-SHA refs
    let is_sha = ref_.is_some_and(looks_like_sha);
    if let Some(r) = ref_ {
        if !is_sha {
            args.push("--branch");
            args.push(r);
        }
    }

    // Sparse clone for subpath (requires git ≥ 2.25)
    let use_sparse = subpath.is_some() && git_supports_sparse().await;
    if use_sparse {
        args.push("--filter=blob:none");
        args.push("--sparse");
    }

    args.push(url);
    let clone_dir_str = clone_dir.to_string_lossy().to_string();
    args.push(&clone_dir_str);

    ui::info(&format!("Cloning from {url}..."));
    let output = run_git(&args, None, 60).await.ok()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        ui::warn(&format!("Git clone failed: {}", stderr.trim()));
        return None;
    }

    // Handle SHA refs: checkout after clone
    if is_sha {
        if let Some(sha) = ref_ {
            let fetch_out = run_git(
                &["fetch", "origin", sha, "--depth", "1"],
                Some(&clone_dir),
                30,
            )
            .await
            .ok()?;
            if !fetch_out.status.success() {
                return None;
            }
            let co_out = run_git(&["checkout", "FETCH_HEAD"], Some(&clone_dir), 10)
                .await
                .ok()?;
            if !co_out.status.success() {
                return None;
            }
        }
    }

    // Set up sparse checkout for subpath
    if let Some(sp) = subpath {
        if use_sparse {
            let out =
                run_git(&["sparse-checkout", "set", sp], Some(&clone_dir), 30)
                    .await
                    .ok()?;
            if !out.status.success() {
                // sparse-checkout failed; fall through to full-tree copy+filter
                ui::warn("Sparse checkout failed, using full clone");
            }
        }
    }

    // Determine source directory
    let source_dir = if let Some(sp) = subpath {
        let candidate = clone_dir.join(sp);
        if candidate.is_dir() {
            candidate
        } else {
            ui::warn(&format!("Subpath '{sp}' not found in cloned repo"));
            return None;
        }
    } else {
        clone_dir.clone()
    };

    // Copy to dest (excluding .git)
    std::fs::create_dir_all(dest).ok()?;
    let files = copy_dir_excluding_git(&source_dir, dest).ok()?;

    // Clean up temp directory
    std::fs::remove_dir_all(&tmp_dir).ok();

    Some(files)
}

/// Extract host from SSH URL: `git@github.com:owner/repo.git` → `github.com`
fn extract_ssh_host(ssh_url: &str) -> Option<String> {
    let after_at = ssh_url.strip_prefix("git@")?;
    let host = after_at.split(':').next()?;
    Some(host.to_string())
}

/// Quick SSH connectivity probe (3-second timeout).
async fn ssh_probe(host: &str) -> bool {
    let result = tokio::process::Command::new("ssh")
        .args([
            "-o",
            "ConnectTimeout=3",
            "-o",
            "StrictHostKeyChecking=accept-new",
            "-o",
            "BatchMode=yes",
            "-T",
            &format!("git@{host}"),
        ])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn();

    let child = match result {
        Ok(c) => c,
        Err(_) => return false,
    };

    // SSH -T to GitHub/GitLab returns exit code 1 but prints a welcome message.
    // Any response (even exit 1) means SSH connectivity works.
    matches!(
        tokio::time::timeout(std::time::Duration::from_secs(5), child.wait_with_output()).await,
        Ok(Ok(_))
    )
}

// ---------------------------------------------------------------------------
// Archive tarball
// ---------------------------------------------------------------------------

/// Download a tarball and extract it, optionally filtering to a subpath.
///
/// Returns `Some(files)` on success, `None` on failure.
pub async fn try_fetch_tarball(
    tarball_url: &str,
    subpath: Option<&str>,
    dest: &Path,
    auth_header: Option<(&str, &str)>,
) -> Option<Vec<PathBuf>> {
    let client = reqwest::Client::builder()
        .user_agent("skillx/0.5")
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()
        .ok()?;

    let mut req = client.get(tarball_url);
    if let Some((key, val)) = auth_header {
        req = req.header(key, val);
    }

    ui::info(&format!("Downloading archive from {tarball_url}..."));

    let resp = req.send().await.ok()?;
    if !resp.status().is_success() {
        ui::warn(&format!(
            "Archive download failed: HTTP {}",
            resp.status()
        ));
        return None;
    }

    let bytes = resp.bytes().await.ok()?;

    if let Some(sub) = subpath {
        // Extract to temp dir, then copy subpath
        let tmp_dir = Config::cache_dir()
            .ok()?
            .join(format!("tmp-archive-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&tmp_dir).ok()?;
        super::archive::ArchiveSource::extract_tar_gz(&bytes, &tmp_dir).ok()?;

        let source_dir = tmp_dir.join(sub);
        if !source_dir.is_dir() {
            ui::warn(&format!("Subpath '{sub}' not found in archive"));
            std::fs::remove_dir_all(&tmp_dir).ok();
            return None;
        }

        std::fs::create_dir_all(dest).ok()?;
        let files = copy_dir_contents(&source_dir, dest).ok()?;
        std::fs::remove_dir_all(&tmp_dir).ok();
        Some(files)
    } else {
        std::fs::create_dir_all(dest).ok()?;
        let files = super::archive::ArchiveSource::extract_tar_gz(&bytes, dest).ok()?;
        Some(files)
    }
}

// ---------------------------------------------------------------------------
// API retry helper
// ---------------------------------------------------------------------------

/// Send an HTTP request with retry on 429/5xx errors.
///
/// `build_request` is called on each attempt (RequestBuilder is consumed by send).
/// Retries up to `max_retries` times with exponential backoff (1s → 2s → 4s).
pub async fn request_with_retry<F>(
    build_request: F,
    max_retries: u32,
) -> Result<reqwest::Response>
where
    F: Fn() -> reqwest::RequestBuilder,
{
    let mut attempt = 0;

    loop {
        let resp = build_request()
            .send()
            .await
            .map_err(|e| SkillxError::Network(format!("request failed: {e}")))?;

        let status = resp.status().as_u16();

        // Check if this is a rate-limit response
        let is_rate_limited = status == 429
            || (status == 403
                && resp
                    .headers()
                    .get("x-ratelimit-remaining")
                    .and_then(|v| v.to_str().ok())
                    .map(|v| v == "0")
                    .unwrap_or(false));

        let is_server_error = (500..600).contains(&status);

        if !is_rate_limited && !is_server_error {
            return Ok(resp);
        }

        attempt += 1;
        if attempt > max_retries {
            if is_rate_limited {
                return Err(SkillxError::RateLimited(format!(
                    "rate limit exceeded after {max_retries} retries (HTTP {status})"
                )));
            }
            return Ok(resp);
        }

        // Determine wait time
        let wait = if is_rate_limited {
            parse_rate_limit_wait(resp.headers())
                .unwrap_or_else(|| backoff_duration(attempt))
        } else {
            backoff_duration(attempt)
        };

        ui::warn(&format!(
            "HTTP {status} — retrying in {}s (attempt {attempt}/{max_retries})...",
            wait.as_secs()
        ));
        tokio::time::sleep(wait).await;
    }
}

/// Parse rate-limit headers to determine how long to wait.
fn parse_rate_limit_wait(headers: &reqwest::header::HeaderMap) -> Option<std::time::Duration> {
    // Check Retry-After (seconds)
    if let Some(val) = headers.get("retry-after") {
        if let Ok(secs) = val.to_str().unwrap_or("").parse::<u64>() {
            return Some(std::time::Duration::from_secs(secs.min(300)));
        }
    }
    // Check x-ratelimit-reset (Unix timestamp)
    if let Some(val) = headers.get("x-ratelimit-reset") {
        if let Ok(reset_ts) = val.to_str().unwrap_or("").parse::<i64>() {
            let now = chrono::Utc::now().timestamp();
            let wait = (reset_ts - now).clamp(1, 300);
            return Some(std::time::Duration::from_secs(wait as u64));
        }
    }
    None
}

/// Exponential backoff: 1s, 2s, 4s, ...
fn backoff_duration(attempt: u32) -> std::time::Duration {
    let secs = 1u64 << (attempt - 1).min(4); // cap at 16s
    std::time::Duration::from_secs(secs)
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Check if `git` is available on PATH.
pub fn is_git_available() -> bool {
    which::which("git").is_ok()
}

/// Check if the installed git version supports sparse checkout (≥ 2.25).
async fn git_supports_sparse() -> bool {
    match git_version().await {
        Some((major, minor)) => major > 2 || (major == 2 && minor >= 25),
        None => false,
    }
}

/// Parse git version from `git --version` output.
pub async fn git_version() -> Option<(u32, u32)> {
    let output = run_git(&["--version"], None, 5).await.ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    parse_git_version(&text)
}

/// Parse version tuple from `git version X.Y.Z` text.
fn parse_git_version(text: &str) -> Option<(u32, u32)> {
    // "git version 2.39.2" → (2, 39)
    let version_str = text.split_whitespace().find(|s| s.contains('.'))?;
    let mut parts = version_str.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    Some((major, minor))
}

/// Execute a git command with a timeout.
async fn run_git(args: &[&str], cwd: Option<&Path>, timeout_secs: u64) -> std::io::Result<Output> {
    let mut cmd = tokio::process::Command::new("git");
    cmd.args(args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);

    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }

    // Disable interactive prompts
    cmd.env("GIT_TERMINAL_PROMPT", "0");

    let child = cmd.spawn()?;

    match tokio::time::timeout(
        std::time::Duration::from_secs(timeout_secs),
        child.wait_with_output(),
    )
    .await
    {
        Ok(result) => result,
        Err(_) => Err(std::io::Error::new(
            std::io::ErrorKind::TimedOut,
            format!("git command timed out after {timeout_secs}s"),
        )),
    }
}

/// Check if a string looks like a git commit SHA (40 hex chars).
pub fn looks_like_sha(s: &str) -> bool {
    s.len() == 40 && s.chars().all(|c| c.is_ascii_hexdigit())
}

/// Recursively copy a directory, excluding `.git/`.
pub fn copy_dir_excluding_git(src: &Path, dst: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    std::fs::create_dir_all(dst)
        .map_err(|e| SkillxError::Source(format!("failed to create dir: {e}")))?;

    let entries = std::fs::read_dir(src)
        .map_err(|e| SkillxError::Source(format!("failed to read dir: {e}")))?;

    for entry in entries {
        let entry = entry.map_err(|e| SkillxError::Source(format!("dir entry error: {e}")))?;
        let name = entry.file_name();
        if name == ".git" {
            continue;
        }

        let src_path = entry.path();
        let dst_path = dst.join(&name);
        let ft = entry
            .file_type()
            .map_err(|e| SkillxError::Source(format!("file type error: {e}")))?;

        if ft.is_symlink() {
            continue; // skip symlinks for security
        } else if ft.is_dir() {
            let sub = copy_dir_excluding_git(&src_path, &dst_path)?;
            files.extend(sub);
        } else {
            std::fs::copy(&src_path, &dst_path).map_err(|e| {
                SkillxError::Source(format!(
                    "failed to copy {} → {}: {e}",
                    src_path.display(),
                    dst_path.display()
                ))
            })?;
            files.push(dst_path);
        }
    }

    Ok(files)
}

/// Recursively copy directory contents, returning list of copied files.
/// (Shared helper also used by sourcehut.rs for subpath extraction.)
pub fn copy_dir_contents(src: &Path, dest: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in std::fs::read_dir(src)
        .map_err(|e| SkillxError::Source(format!("failed to read dir: {e}")))?
    {
        let entry = entry.map_err(|e| SkillxError::Source(format!("failed to read entry: {e}")))?;
        let file_type = entry
            .file_type()
            .map_err(|e| SkillxError::Source(format!("failed to get file type: {e}")))?;

        if file_type.is_symlink() {
            continue;
        }

        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());

        if file_type.is_dir() {
            std::fs::create_dir_all(&dest_path)
                .map_err(|e| SkillxError::Source(format!("failed to create dir: {e}")))?;
            let sub_files = copy_dir_contents(&src_path, &dest_path)?;
            files.extend(sub_files);
        } else {
            std::fs::copy(&src_path, &dest_path)
                .map_err(|e| SkillxError::Source(format!("failed to copy file: {e}")))?;
            files.push(dest_path);
        }
    }

    Ok(files)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_looks_like_sha() {
        assert!(looks_like_sha(
            "abcd1234567890abcdef1234567890abcdef1234"
        ));
        assert!(looks_like_sha(
            "0000000000000000000000000000000000000000"
        ));
        // Too short
        assert!(!looks_like_sha("abc123"));
        // Not hex
        assert!(!looks_like_sha(
            "xyzd1234567890abcdef1234567890abcdef1234"
        ));
        // Branch name
        assert!(!looks_like_sha("main"));
        assert!(!looks_like_sha("v1.0"));
    }

    #[test]
    fn test_parse_git_version() {
        assert_eq!(
            parse_git_version("git version 2.39.2"),
            Some((2, 39))
        );
        assert_eq!(
            parse_git_version("git version 2.25.0 (Apple Git-100)"),
            Some((2, 25))
        );
        assert_eq!(
            parse_git_version("git version 1.8.3.1"),
            Some((1, 8))
        );
        assert_eq!(parse_git_version("not a version"), None);
    }

    #[test]
    fn test_extract_ssh_host() {
        assert_eq!(
            extract_ssh_host("git@github.com:owner/repo.git"),
            Some("github.com".to_string())
        );
        assert_eq!(
            extract_ssh_host("git@gitlab.example.com:org/repo.git"),
            Some("gitlab.example.com".to_string())
        );
        assert_eq!(extract_ssh_host("https://github.com/o/r"), None);
    }

    #[test]
    fn test_is_git_available() {
        // CI environments always have git installed
        assert!(is_git_available());
    }

    #[test]
    fn test_copy_dir_excluding_git() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src");
        let dst = tmp.path().join("dst");

        // Create source with .git directory and regular files
        std::fs::create_dir_all(src.join(".git/objects")).unwrap();
        std::fs::write(src.join(".git/HEAD"), "ref: refs/heads/main").unwrap();
        std::fs::write(src.join("SKILL.md"), "# Test").unwrap();
        std::fs::create_dir_all(src.join("sub")).unwrap();
        std::fs::write(src.join("sub/file.txt"), "hello").unwrap();

        let files = copy_dir_excluding_git(&src, &dst).unwrap();

        // .git should be excluded
        assert!(!dst.join(".git").exists());
        // Regular files should be copied
        assert!(dst.join("SKILL.md").exists());
        assert!(dst.join("sub/file.txt").exists());
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_copy_dir_contents() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src");
        let dst = tmp.path().join("dst");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::create_dir_all(&dst).unwrap();

        std::fs::write(src.join("a.md"), "# A").unwrap();
        std::fs::create_dir_all(src.join("dir")).unwrap();
        std::fs::write(src.join("dir/b.txt"), "B").unwrap();

        let files = copy_dir_contents(&src, &dst).unwrap();
        assert_eq!(files.len(), 2);
        assert!(dst.join("a.md").exists());
        assert!(dst.join("dir/b.txt").exists());
    }

    #[test]
    fn test_backoff_duration() {
        assert_eq!(backoff_duration(1), std::time::Duration::from_secs(1));
        assert_eq!(backoff_duration(2), std::time::Duration::from_secs(2));
        assert_eq!(backoff_duration(3), std::time::Duration::from_secs(4));
        assert_eq!(backoff_duration(4), std::time::Duration::from_secs(8));
        // Capped at 16s
        assert_eq!(backoff_duration(5), std::time::Duration::from_secs(16));
        assert_eq!(backoff_duration(10), std::time::Duration::from_secs(16));
    }
}

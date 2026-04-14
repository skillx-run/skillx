use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::error::{Result, SkillxError};
use crate::ui;

pub struct SourceHutSource;

impl SourceHutSource {
    /// Fetch a skill from SourceHut using a two-tier strategy:
    ///   1. Tarball download with platform-specific error handling
    ///   2. Git clone (HTTPS first, SSH fallback)
    pub async fn fetch(
        owner: &str,
        repo: &str,
        path: Option<&str>,
        ref_: Option<&str>,
        dest: &Path,
    ) -> Result<Vec<PathBuf>> {
        let ref_name = ref_.unwrap_or("HEAD");

        // Tier 1: Tarball download with precise error handling
        match Self::fetch_tarball(owner, repo, path, ref_name, dest).await {
            Ok(files) => return Ok(files),
            Err(e) => {
                // Propagate auth/permission/not-found errors directly
                match &e {
                    SkillxError::SourceHutApi(_) => return Err(e),
                    _ => {
                        ui::warn(&format!("Tarball download failed: {e}"));
                    }
                }
            }
        }

        // Tier 2: Git clone
        let https_url = format!("https://git.sr.ht/~{owner}/{repo}");
        let ssh_url = format!("git@git.sr.ht:~{owner}/{repo}");

        if let Some(files) =
            super::git_clone::clone_skill(&https_url, Some(&ssh_url), path, ref_, dest).await
        {
            return Ok(files);
        }

        Err(SkillxError::SourceHutApi(format!(
            "failed to fetch ~{owner}/{repo}. Check the repository and ref, or set SRHT_TOKEN for private repos."
        )))
    }

    /// Download and extract tarball with platform-specific error messages.
    async fn fetch_tarball(
        owner: &str,
        repo: &str,
        path: Option<&str>,
        ref_name: &str,
        dest: &Path,
    ) -> Result<Vec<PathBuf>> {
        let tarball_url = format!("https://git.sr.ht/~{owner}/{repo}/archive/{ref_name}.tar.gz");

        let client = reqwest::Client::builder()
            .user_agent("skillx/0.5")
            .redirect(reqwest::redirect::Policy::limited(10))
            .build()
            .map_err(|e| SkillxError::Network(format!("failed to create HTTP client: {e}")))?;

        let token = std::env::var("SRHT_TOKEN").ok();

        let mut req = client.get(&tarball_url);
        if let Some(ref t) = token {
            req = req.header("Authorization", format!("Bearer {t}"));
        }

        ui::info(&format!("Downloading archive from {tarball_url}..."));

        let resp = req
            .send()
            .await
            .map_err(|e| SkillxError::Network(format!("SourceHut tarball download failed: {e}")))?;

        match resp.status().as_u16() {
            401 => {
                return Err(SkillxError::SourceHutApi(
                    "authentication required. Set SRHT_TOKEN environment variable.".into(),
                ));
            }
            403 => {
                return Err(SkillxError::SourceHutApi(
                    "access denied. Repository may be private — set SRHT_TOKEN.".into(),
                ));
            }
            404 => {
                return Err(SkillxError::SourceHutApi(
                    "not found. Check the owner, repository, and ref.".into(),
                ));
            }
            s if !(200..300).contains(&s) => {
                return Err(SkillxError::SourceHutApi(format!(
                    "SourceHut returned HTTP {s}"
                )));
            }
            _ => {}
        }

        let bytes = resp
            .bytes()
            .await
            .map_err(|e| SkillxError::Network(format!("failed to read SourceHut tarball: {e}")))?;

        if let Some(sub_path) = path {
            // Extract to temp dir, then copy the target sub-path to dest
            let tmp_dir = Config::cache_dir()?.join(format!("tmp-{}", uuid::Uuid::new_v4()));
            std::fs::create_dir_all(&tmp_dir)
                .map_err(|e| SkillxError::Source(format!("failed to create temp dir: {e}")))?;

            super::archive::ArchiveSource::extract_tar_gz(&bytes, &tmp_dir)?;

            let source_dir = tmp_dir.join(sub_path);
            if !source_dir.exists() {
                std::fs::remove_dir_all(&tmp_dir).ok();
                return Err(SkillxError::SourceHutApi(format!(
                    "path '{sub_path}' not found in SourceHut repo ~{owner}/{repo}"
                )));
            }

            std::fs::create_dir_all(dest)
                .map_err(|e| SkillxError::Source(format!("failed to create dest dir: {e}")))?;

            let files = super::git_clone::copy_dir_contents(&source_dir, dest)?;
            std::fs::remove_dir_all(&tmp_dir).ok();
            Ok(files)
        } else {
            std::fs::create_dir_all(dest)
                .map_err(|e| SkillxError::Source(format!("failed to create dest dir: {e}")))?;
            super::archive::ArchiveSource::extract_tar_gz(&bytes, dest)
        }
    }
}

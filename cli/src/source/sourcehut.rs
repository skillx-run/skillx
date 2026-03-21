use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::error::{Result, SkillxError};

pub struct SourceHutSource;

impl SourceHutSource {
    /// Fetch a skill from SourceHut via tarball download.
    ///
    /// Uses `https://git.sr.ht/~{owner}/{repo}/archive/{ref}.tar.gz` and
    /// extracts via the existing archive tar.gz extraction.
    pub async fn fetch(
        owner: &str,
        repo: &str,
        path: Option<&str>,
        ref_: Option<&str>,
        dest: &Path,
    ) -> Result<Vec<PathBuf>> {
        let ref_name = ref_.unwrap_or("HEAD");
        let tarball_url = format!("https://git.sr.ht/~{owner}/{repo}/archive/{ref_name}.tar.gz");

        let client = reqwest::Client::builder()
            .user_agent("skillx/0.3")
            .build()
            .map_err(|e| SkillxError::Network(format!("failed to create HTTP client: {e}")))?;

        let token = std::env::var("SRHT_TOKEN").ok();

        let mut req = client.get(&tarball_url);
        if let Some(ref t) = token {
            req = req.header("Authorization", format!("Bearer {t}"));
        }

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

            // Find the sub_path within the extracted content
            let source_dir = tmp_dir.join(sub_path);
            if !source_dir.exists() {
                // Clean up
                std::fs::remove_dir_all(&tmp_dir).ok();
                return Err(SkillxError::SourceHutApi(format!(
                    "path '{sub_path}' not found in SourceHut repo ~{owner}/{repo}"
                )));
            }

            std::fs::create_dir_all(dest)
                .map_err(|e| SkillxError::Source(format!("failed to create dest dir: {e}")))?;

            let files = copy_dir_contents(&source_dir, dest)?;

            // Clean up temp dir
            std::fs::remove_dir_all(&tmp_dir).ok();

            Ok(files)
        } else {
            // Extract directly to dest
            std::fs::create_dir_all(dest)
                .map_err(|e| SkillxError::Source(format!("failed to create dest dir: {e}")))?;
            super::archive::ArchiveSource::extract_tar_gz(&bytes, dest)
        }
    }
}

/// Recursively copy directory contents, returning list of copied files.
fn copy_dir_contents(src: &Path, dest: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in std::fs::read_dir(src)
        .map_err(|e| SkillxError::Source(format!("failed to read dir: {e}")))?
    {
        let entry = entry.map_err(|e| SkillxError::Source(format!("failed to read entry: {e}")))?;
        let file_type = entry
            .file_type()
            .map_err(|e| SkillxError::Source(format!("failed to get file type: {e}")))?;

        // Skip symlinks
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

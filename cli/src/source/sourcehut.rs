use std::path::{Path, PathBuf};

use crate::error::{Result, SkillxError};

pub struct SourceHutSource;

impl SourceHutSource {
    /// Fetch a skill from SourceHut using a two-tier strategy:
    ///   1. Tarball download (existing approach, most reliable for SourceHut)
    ///   2. Git clone (HTTPS first, SSH fallback)
    pub async fn fetch(
        owner: &str,
        repo: &str,
        path: Option<&str>,
        ref_: Option<&str>,
        dest: &Path,
    ) -> Result<Vec<PathBuf>> {
        let ref_name = ref_.unwrap_or("HEAD");

        // Tier 1: Tarball download (SourceHut's native mechanism)
        let tarball_url = format!("https://git.sr.ht/~{owner}/{repo}/archive/{ref_name}.tar.gz");
        let auth = std::env::var("SRHT_TOKEN")
            .ok()
            .map(|t| ("Authorization".to_string(), format!("Bearer {t}")));
        let auth_ref = auth
            .as_ref()
            .map(|(k, v)| (k.as_str(), v.as_str()));

        if let Some(files) =
            super::git_clone::try_fetch_tarball(&tarball_url, path, dest, auth_ref).await
        {
            return Ok(files);
        }

        // Tier 2: Git clone
        let https_url = format!("https://git.sr.ht/~{owner}/{repo}");
        let ssh_url = format!("git@git.sr.ht:~{owner}/{repo}");

        if let Some(files) =
            super::git_clone::clone_skill(&https_url, Some(&ssh_url), path, ref_, dest).await
        {
            return Ok(files);
        }

        // Both tiers failed — return an error
        Err(SkillxError::SourceHutApi(format!(
            "failed to fetch ~{owner}/{repo}. Check the repository and ref, or set SRHT_TOKEN for private repos."
        )))
    }
}

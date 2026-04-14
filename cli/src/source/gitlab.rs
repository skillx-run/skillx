use std::path::{Path, PathBuf};

use crate::error::{Result, SkillxError};
use crate::source::SkillSource;
use crate::ui;

pub struct GitLabSource;

impl GitLabSource {
    /// Parse a GitLab URL like `https://gitlab.com/owner/repo/-/tree/ref/path`.
    pub fn parse_url(url: &str) -> Result<SkillSource> {
        // Delegate to url.rs — this is kept for API symmetry with GitHubSource
        crate::source::url::resolve_url(url)
    }

    /// Fetch a skill from a GitLab instance using a three-tier strategy.
    ///
    /// Tier order adapts: subpath → git sparse clone first; whole repo → tarball first.
    pub async fn fetch(
        host: &str,
        owner: &str,
        repo: &str,
        path: Option<&str>,
        ref_: Option<&str>,
        dest: &Path,
    ) -> Result<Vec<PathBuf>> {
        let ref_name = ref_.unwrap_or("HEAD");
        let tarball_url =
            format!("https://{host}/{owner}/{repo}/-/archive/{ref_name}/{repo}-{ref_name}.tar.gz");
        let auth = std::env::var("GITLAB_TOKEN")
            .ok()
            .map(|t| ("PRIVATE-TOKEN".to_string(), t));
        let auth_ref = auth.as_ref().map(|(k, v)| (k.as_str(), v.as_str()));
        let https_url = format!("https://{host}/{owner}/{repo}.git");
        let ssh_url = format!("git@{host}:{owner}/{repo}.git");

        if path.is_some() {
            if let Some(files) =
                super::git_clone::clone_skill(&https_url, Some(&ssh_url), path, ref_, dest).await
            {
                return Ok(files);
            }
            if let Some(files) =
                super::git_clone::try_fetch_tarball(&tarball_url, path, dest, auth_ref).await
            {
                return Ok(files);
            }
        } else {
            if let Some(files) =
                super::git_clone::try_fetch_tarball(&tarball_url, path, dest, auth_ref).await
            {
                return Ok(files);
            }
            if let Some(files) =
                super::git_clone::clone_skill(&https_url, Some(&ssh_url), path, ref_, dest).await
            {
                return Ok(files);
            }
        }

        ui::info("Falling back to GitLab API...");
        Self::fetch_via_api(host, owner, repo, path, ref_, dest).await
    }

    /// Fetch via GitLab Repository Files API (fallback with retry).
    async fn fetch_via_api(
        host: &str,
        owner: &str,
        repo: &str,
        path: Option<&str>,
        ref_: Option<&str>,
        dest: &Path,
    ) -> Result<Vec<PathBuf>> {
        let client = reqwest::Client::builder()
            .user_agent("skillx/0.5")
            .build()
            .map_err(|e| SkillxError::Network(format!("failed to create HTTP client: {e}")))?;

        let token = std::env::var("GITLAB_TOKEN").ok();

        // Project ID via URL-encoded owner/repo
        let project_id = super::urlencoding(&format!("{owner}/{repo}"));

        // List files in the directory
        let api_path = path.unwrap_or("");
        let ref_param = ref_.unwrap_or("HEAD");
        let tree_url = format!(
            "https://{host}/api/v4/projects/{project_id}/repository/tree?path={}&ref={}&per_page=100&recursive=true",
            super::urlencode_path(api_path),
            super::urlencoding(ref_param),
        );

        let token_clone = token.clone();
        let resp = super::git_clone::request_with_retry(
            || {
                let mut req = client.get(&tree_url);
                if let Some(ref t) = token_clone {
                    req = req.header("PRIVATE-TOKEN", t.as_str());
                }
                req
            },
            3,
        )
        .await?;

        match resp.status().as_u16() {
            401 => {
                return Err(SkillxError::GitLabApi(
                    "authentication required. Set GITLAB_TOKEN environment variable.".into(),
                ));
            }
            403 => {
                return Err(SkillxError::GitLabApi(
                    "access denied. Repository may be private — set GITLAB_TOKEN.".into(),
                ));
            }
            404 => {
                return Err(SkillxError::GitLabApi(
                    "not found. Check the owner, repository, and path.".into(),
                ));
            }
            s if !(200..300).contains(&s) => {
                return Err(SkillxError::GitLabApi(format!(
                    "GitLab API returned HTTP {s}"
                )));
            }
            _ => {}
        }

        let items: Vec<serde_json::Value> = resp.json().await.map_err(|e| {
            SkillxError::GitLabApi(format!("failed to parse GitLab tree response: {e}"))
        })?;

        std::fs::create_dir_all(dest)
            .map_err(|e| SkillxError::Source(format!("failed to create dir: {e}")))?;

        // Download all files concurrently
        use futures::stream::{FuturesUnordered, StreamExt};

        let futures: FuturesUnordered<_> = items
            .iter()
            .filter_map(|item| {
                let item_type = item["type"].as_str()?;
                if item_type != "blob" {
                    return None;
                }
                let file_path = item["path"].as_str()?;
                // Compute relative path from the skill root using directory-boundary match
                let relative = if !api_path.is_empty() {
                    let prefix_with_slash = format!("{api_path}/");
                    file_path
                        .strip_prefix(&prefix_with_slash)
                        .unwrap_or(file_path)
                } else {
                    file_path
                };
                let dest_path = dest.join(relative);
                let raw_url = format!(
                    "https://{host}/api/v4/projects/{project_id}/repository/files/{}/raw?ref={}",
                    super::urlencode_path(file_path),
                    super::urlencoding(ref_param),
                );
                let client = client.clone();
                let token = token.clone();
                let name = file_path.to_string();
                Some(async move {
                    let resp = super::git_clone::request_with_retry(
                        || {
                            let mut req = client.get(&raw_url);
                            if let Some(ref t) = token {
                                req = req.header("PRIVATE-TOKEN", t.as_str());
                            }
                            req
                        },
                        3,
                    )
                    .await?;
                    if !resp.status().is_success() {
                        return Err(SkillxError::GitLabApi(format!(
                            "download failed for {name}: HTTP {}",
                            resp.status()
                        )));
                    }
                    let bytes = resp
                        .bytes()
                        .await
                        .map_err(|e| SkillxError::Network(format!("failed to read {name}: {e}")))?;
                    if let Some(parent) = dest_path.parent() {
                        std::fs::create_dir_all(parent).map_err(|e| {
                            SkillxError::Source(format!(
                                "failed to create dir {}: {e}",
                                parent.display()
                            ))
                        })?;
                    }
                    std::fs::write(&dest_path, &bytes).map_err(|e| {
                        SkillxError::Source(format!("failed to write {}: {e}", dest_path.display()))
                    })?;
                    Ok::<PathBuf, SkillxError>(dest_path)
                })
            })
            .collect();

        let results: Vec<_> = futures.collect().await;
        let mut downloaded = Vec::new();
        for r in results {
            downloaded.push(r?);
        }

        Ok(downloaded)
    }
}

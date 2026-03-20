use std::path::{Path, PathBuf};

use crate::error::{Result, SkillxError};
use crate::source::SkillSource;

pub struct GitLabSource;

impl GitLabSource {
    /// Parse a GitLab URL like `https://gitlab.com/owner/repo/-/tree/ref/path`.
    pub fn parse_url(url: &str) -> Result<SkillSource> {
        // Delegate to url.rs — this is kept for API symmetry with GitHubSource
        crate::source::url::resolve_url(url)
    }

    /// Fetch a skill from a GitLab instance using the Repository Files API.
    ///
    /// API: GET /api/v4/projects/:id/repository/tree?path=&ref=
    ///      GET /api/v4/projects/:id/repository/files/:path/raw?ref=
    pub async fn fetch(
        host: &str,
        owner: &str,
        repo: &str,
        path: Option<&str>,
        ref_: Option<&str>,
        dest: &Path,
    ) -> Result<Vec<PathBuf>> {
        let client = reqwest::Client::builder()
            .user_agent("skillx/0.2")
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

        let mut req = client.get(&tree_url);
        if let Some(ref t) = token {
            req = req.header("PRIVATE-TOKEN", t.as_str());
        }

        let resp = req.send().await.map_err(|e| {
            SkillxError::Network(format!("GitLab API request failed: {e}"))
        })?;

        if !resp.status().is_success() {
            return Err(SkillxError::GitLabApi(format!(
                "GitLab API returned {} for {}",
                resp.status(),
                tree_url
            )));
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
                // Compute relative path from the skill root
                let relative = if let Some(prefix) = api_path.strip_suffix('/') {
                    file_path.strip_prefix(prefix).unwrap_or(file_path)
                } else if !api_path.is_empty() {
                    file_path
                        .strip_prefix(api_path)
                        .and_then(|p| p.strip_prefix('/'))
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
                    let mut req = client.get(&raw_url);
                    if let Some(ref t) = token {
                        req = req.header("PRIVATE-TOKEN", t.as_str());
                    }
                    let resp = req.send().await.map_err(|e| {
                        SkillxError::Network(format!("download failed for {name}: {e}"))
                    })?;
                    if !resp.status().is_success() {
                        return Err(SkillxError::GitLabApi(format!(
                            "download failed for {name}: HTTP {}",
                            resp.status()
                        )));
                    }
                    let bytes = resp.bytes().await.map_err(|e| {
                        SkillxError::Network(format!("failed to read {name}: {e}"))
                    })?;
                    if let Some(parent) = dest_path.parent() {
                        std::fs::create_dir_all(parent).map_err(|e| {
                            SkillxError::Source(format!(
                                "failed to create dir {}: {e}",
                                parent.display()
                            ))
                        })?;
                    }
                    std::fs::write(&dest_path, &bytes).map_err(|e| {
                        SkillxError::Source(format!(
                            "failed to write {}: {e}",
                            dest_path.display()
                        ))
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

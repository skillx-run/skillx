use std::path::{Path, PathBuf};

use crate::error::{Result, SkillxError};
use crate::source::SkillSource;

pub struct GiteaSource;

/// Context for Gitea API operations to avoid excessive function arguments.
struct FetchContext {
    client: reqwest::Client,
    host: String,
    owner: String,
    repo: String,
    ref_: Option<String>,
    root_path: String,
    token: Option<String>,
}

impl GiteaSource {
    /// Parse a Gitea/Codeberg URL.
    pub fn parse_url(url: &str) -> Result<SkillSource> {
        crate::source::url::resolve_url(url)
    }

    /// Fetch a skill from a Gitea/Forgejo/Codeberg instance.
    ///
    /// API: GET /api/v1/repos/:owner/:repo/contents/:path?ref=
    /// Returns JSON array with `download_url` for each file.
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

        let token = std::env::var("GITEA_TOKEN").ok();
        let api_path = path.unwrap_or("");

        let ctx = FetchContext {
            client,
            host: host.to_string(),
            owner: owner.to_string(),
            repo: repo.to_string(),
            ref_: ref_.map(|s| s.to_string()),
            root_path: api_path.to_string(),
            token,
        };

        std::fs::create_dir_all(dest)
            .map_err(|e| SkillxError::Source(format!("failed to create dir: {e}")))?;

        Self::fetch_dir(&ctx, api_path, dest).await
    }

    /// Recursively fetch a directory from a Gitea instance.
    async fn fetch_dir(
        ctx: &FetchContext,
        path: &str,
        dest: &Path,
    ) -> Result<Vec<PathBuf>> {
        let mut url = format!(
            "https://{}/api/v1/repos/{}/{}/contents/{path}",
            ctx.host, ctx.owner, ctx.repo,
        );
        if let Some(ref r) = ctx.ref_ {
            url.push_str(&format!("?ref={}", super::urlencoding(r)));
        }

        let mut req = ctx.client.get(&url);
        if let Some(ref t) = ctx.token {
            req = req.header("Authorization", format!("token {t}"));
        }

        let resp = req.send().await.map_err(|e| {
            SkillxError::Network(format!("Gitea API request failed: {e}"))
        })?;

        if !resp.status().is_success() {
            return Err(SkillxError::GiteaApi(format!(
                "Gitea API returned {} for {}/{}/{}/{}",
                resp.status(),
                ctx.host, ctx.owner, ctx.repo, path
            )));
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| {
            SkillxError::GiteaApi(format!("failed to parse Gitea response: {e}"))
        })?;

        let mut downloaded = Vec::new();

        // Gitea contents API returns either an array (directory) or an object (file)
        let items = if let Some(arr) = body.as_array() {
            arr.clone()
        } else {
            vec![body]
        };

        use futures::stream::{FuturesUnordered, StreamExt};

        let file_futures: FuturesUnordered<_> = items
            .iter()
            .filter_map(|item| {
                let item_type = item["type"].as_str()?;
                if item_type != "file" {
                    return None;
                }
                let download_url = item["download_url"].as_str()?;
                let file_path = item["path"].as_str()?;
                let relative = strip_root_prefix(file_path, &ctx.root_path);
                let dest_path = dest.join(relative);
                let url = download_url.to_string();
                let client = ctx.client.clone();
                let token = ctx.token.clone();
                let name = file_path.to_string();
                Some(async move {
                    let mut req = client.get(&url);
                    if let Some(ref t) = token {
                        req = req.header("Authorization", format!("token {t}"));
                    }
                    let resp = req.send().await.map_err(|e| {
                        SkillxError::Network(format!("download failed for {name}: {e}"))
                    })?;
                    if !resp.status().is_success() {
                        return Err(SkillxError::GiteaApi(format!(
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

        let results: Vec<_> = file_futures.collect().await;
        for r in results {
            downloaded.push(r?);
        }

        // Recurse into subdirectories
        for item in &items {
            if item["type"].as_str() == Some("dir") {
                if let Some(dir_path) = item["path"].as_str() {
                    let relative = strip_root_prefix(dir_path, &ctx.root_path);
                    let sub_dest = dest.join(relative);
                    let sub_files =
                        Box::pin(Self::fetch_dir(ctx, dir_path, &sub_dest)).await?;
                    downloaded.extend(sub_files);
                }
            }
        }

        Ok(downloaded)
    }
}

/// Strip the root path prefix from a file path to get the relative path.
///
/// Uses directory-boundary matching: `root_path` must match a full path
/// segment (followed by `/`), not just a string prefix.
fn strip_root_prefix<'a>(file_path: &'a str, root_path: &str) -> &'a str {
    if root_path.is_empty() {
        return file_path;
    }
    // Try "root_path/" as prefix (directory boundary)
    let with_slash = format!("{root_path}/");
    if let Some(rest) = file_path.strip_prefix(&with_slash) {
        return rest;
    }
    // Exact match (file_path == root_path, shouldn't happen for files)
    if file_path == root_path {
        return "";
    }
    file_path
}

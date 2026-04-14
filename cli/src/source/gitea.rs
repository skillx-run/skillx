use std::path::{Path, PathBuf};

use crate::error::{Result, SkillxError};
use crate::source::SkillSource;
use crate::ui;

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

    /// Fetch a skill from a Gitea/Forgejo/Codeberg instance using a three-tier strategy.
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
        let tarball_url = format!("https://{host}/{owner}/{repo}/archive/{ref_name}.tar.gz");
        let auth = std::env::var("GITEA_TOKEN")
            .ok()
            .map(|t| ("Authorization".to_string(), format!("token {t}")));
        let auth_ref = auth
            .as_ref()
            .map(|(k, v)| (k.as_str(), v.as_str()));
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

        ui::info("Falling back to Gitea API...");
        Self::fetch_via_api(host, owner, repo, path, ref_, dest).await
    }

    /// Fetch via Gitea Contents API (fallback with retry).
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
    async fn fetch_dir(ctx: &FetchContext, path: &str, dest: &Path) -> Result<Vec<PathBuf>> {
        let mut url = format!(
            "https://{}/api/v1/repos/{}/{}/contents/{path}",
            ctx.host, ctx.owner, ctx.repo,
        );
        if let Some(ref r) = ctx.ref_ {
            url.push_str(&format!("?ref={}", super::urlencoding(r)));
        }

        let token_clone = ctx.token.clone();
        let resp = super::git_clone::request_with_retry(
            || {
                let mut req = ctx.client.get(&url);
                if let Some(ref t) = token_clone {
                    req = req.header("Authorization", format!("token {t}"));
                }
                req
            },
            3,
        )
        .await?;

        match resp.status().as_u16() {
            401 => {
                return Err(SkillxError::GiteaApi(
                    "authentication required. Set GITEA_TOKEN environment variable.".into(),
                ));
            }
            403 => {
                return Err(SkillxError::GiteaApi(
                    "access denied. Repository may be private — set GITEA_TOKEN.".into(),
                ));
            }
            404 => {
                return Err(SkillxError::GiteaApi(
                    "not found. Check the owner, repository, and path.".into(),
                ));
            }
            s if !(200..300).contains(&s) => {
                return Err(SkillxError::GiteaApi(format!(
                    "Gitea API returned HTTP {s}"
                )));
            }
            _ => {}
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| SkillxError::GiteaApi(format!("failed to parse Gitea response: {e}")))?;

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
                    let resp = super::git_clone::request_with_retry(
                        || {
                            let mut req = client.get(&url);
                            if let Some(ref t) = token {
                                req = req.header("Authorization", format!("token {t}"));
                            }
                            req
                        },
                        3,
                    )
                    .await?;
                    if !resp.status().is_success() {
                        return Err(SkillxError::GiteaApi(format!(
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
                    let sub_files = Box::pin(Self::fetch_dir(ctx, dir_path, &sub_dest)).await?;
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

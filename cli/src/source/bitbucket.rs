use std::path::{Path, PathBuf};

use crate::error::{Result, SkillxError};
use crate::source::SkillSource;
use crate::ui;

pub struct BitbucketSource;

/// Context for Bitbucket API operations to avoid excessive function arguments.
struct FetchContext {
    client: reqwest::Client,
    owner: String,
    repo: String,
    ref_: String,
    root_path: String,
    token: Option<String>,
    basic_auth: Option<(String, String)>,
}

impl FetchContext {
    fn auth_request(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(ref t) = self.token {
            req.bearer_auth(t)
        } else if let Some((ref user, ref pass)) = self.basic_auth {
            req.basic_auth(user, Some(pass))
        } else {
            req
        }
    }
}

impl BitbucketSource {
    /// Parse a Bitbucket URL like `https://bitbucket.org/owner/repo/src/ref/path`.
    pub fn parse_url(url: &str) -> Result<SkillSource> {
        crate::source::url::resolve_url(url)
    }

    /// Fetch a skill from Bitbucket using a three-tier strategy:
    ///   1. Archive tarball (no API rate limits)
    ///   2. Git clone (HTTPS first, SSH fallback)
    ///   3. Source API with retry (last resort)
    pub async fn fetch(
        owner: &str,
        repo: &str,
        path: Option<&str>,
        ref_: Option<&str>,
        dest: &Path,
    ) -> Result<Vec<PathBuf>> {
        let ref_name = ref_.unwrap_or("HEAD");

        // Tier 1: Archive tarball
        let tarball_url =
            format!("https://bitbucket.org/{owner}/{repo}/get/{ref_name}.tar.gz");
        let auth = std::env::var("BITBUCKET_TOKEN")
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
        let https_url = format!("https://bitbucket.org/{owner}/{repo}.git");
        let ssh_url = format!("git@bitbucket.org:{owner}/{repo}.git");

        if let Some(files) =
            super::git_clone::clone_skill(&https_url, Some(&ssh_url), path, ref_, dest).await
        {
            return Ok(files);
        }

        // Tier 3: Source API with retry
        ui::info("Falling back to Bitbucket API...");
        Self::fetch_via_api(owner, repo, path, ref_, dest).await
    }

    /// Fetch via Bitbucket Source API (fallback with retry).
    async fn fetch_via_api(
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

        let token = std::env::var("BITBUCKET_TOKEN").ok();
        let basic_auth = std::env::var("BITBUCKET_USERNAME")
            .ok()
            .zip(std::env::var("BITBUCKET_APP_PASSWORD").ok());

        let api_path = path.unwrap_or("");
        let ctx = FetchContext {
            client,
            owner: owner.to_string(),
            repo: repo.to_string(),
            ref_: ref_.unwrap_or("HEAD").to_string(),
            root_path: api_path.to_string(),
            token,
            basic_auth,
        };

        std::fs::create_dir_all(dest)
            .map_err(|e| SkillxError::Source(format!("failed to create dir: {e}")))?;

        Self::fetch_dir(&ctx, api_path, dest).await
    }

    /// Recursively fetch a Bitbucket directory.
    async fn fetch_dir(ctx: &FetchContext, path: &str, dest: &Path) -> Result<Vec<PathBuf>> {
        let url = format!(
            "https://api.bitbucket.org/2.0/repositories/{}/{}/src/{}/{path}",
            ctx.owner, ctx.repo, ctx.ref_,
        );

        let resp = super::git_clone::request_with_retry(
            || ctx.auth_request(ctx.client.get(&url)),
            3,
        )
        .await?;

        match resp.status().as_u16() {
            401 => {
                return Err(SkillxError::BitbucketApi(
                    "authentication required. Set BITBUCKET_TOKEN environment variable.".into(),
                ));
            }
            403 => {
                return Err(SkillxError::BitbucketApi(
                    "access denied. Repository may be private — set BITBUCKET_TOKEN.".into(),
                ));
            }
            404 => {
                return Err(SkillxError::BitbucketApi(
                    "not found. Check the owner, repository, and path.".into(),
                ));
            }
            s if !(200..300).contains(&s) => {
                return Err(SkillxError::BitbucketApi(format!(
                    "Bitbucket API returned HTTP {s}"
                )));
            }
            _ => {}
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| {
            SkillxError::BitbucketApi(format!("failed to parse Bitbucket response: {e}"))
        })?;

        let values = body["values"].as_array().ok_or_else(|| {
            SkillxError::BitbucketApi("unexpected Bitbucket API response format".into())
        })?;

        let mut downloaded = Vec::new();

        use futures::stream::{FuturesUnordered, StreamExt};

        let file_futures: FuturesUnordered<_> = values
            .iter()
            .filter_map(|item| {
                let item_type = item["type"].as_str()?;
                if item_type != "commit_file" {
                    return None;
                }
                let file_path = item["path"].as_str()?;
                let relative = strip_root_prefix(file_path, &ctx.root_path);
                let dest_path = dest.join(relative);
                let download_url = format!(
                    "https://api.bitbucket.org/2.0/repositories/{}/{}/src/{}/{file_path}",
                    ctx.owner, ctx.repo, ctx.ref_,
                );
                let client = ctx.client.clone();
                let token = ctx.token.clone();
                let basic_auth = ctx.basic_auth.clone();
                let name = file_path.to_string();
                Some(async move {
                    let resp = super::git_clone::request_with_retry(
                        || {
                            let mut req = client.get(&download_url);
                            if let Some(ref t) = token {
                                req = req.bearer_auth(t);
                            } else if let Some((ref user, ref pass)) = basic_auth {
                                req = req.basic_auth(user, Some(pass));
                            }
                            req.header("Accept", "application/octet-stream")
                        },
                        3,
                    )
                    .await?;
                    if !resp.status().is_success() {
                        return Err(SkillxError::BitbucketApi(format!(
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
        for item in values {
            if item["type"].as_str() == Some("commit_directory") {
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

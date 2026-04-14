use std::path::{Path, PathBuf};

use crate::error::{Result, SkillxError};
use crate::source::SkillSource;
use crate::ui;

pub struct GitHubSource;

/// Context for GitHub API fetch operations to avoid excessive arguments.
struct ApiFetchContext<'a> {
    client: &'a reqwest::Client,
    token: &'a Option<String>,
    semaphore: &'a std::sync::Arc<tokio::sync::Semaphore>,
    owner: &'a str,
    repo: &'a str,
}

impl GitHubSource {
    /// Parse `owner/repo/path[@ref]` format.
    pub fn parse(input: &str) -> Result<SkillSource> {
        let (main, ref_) = if let Some((main, r)) = input.rsplit_once('@') {
            (main, Some(r.to_string()))
        } else {
            (input, None)
        };

        let parts: Vec<&str> = main.splitn(3, '/').collect();
        if parts.len() < 2 {
            return Err(SkillxError::InvalidSource(format!(
                "invalid github source: '{input}'. Expected: owner/repo[/path][@ref]"
            )));
        }

        let owner = parts[0].to_string();
        let repo = parts[1].to_string();
        let path = if parts.len() > 2 {
            Some(parts[2].to_string())
        } else {
            None
        };

        Ok(SkillSource::GitHub {
            owner,
            repo,
            path,
            ref_,
        })
    }

    /// Parse a GitHub URL like `https://github.com/owner/repo/tree/ref/path`.
    pub fn parse_url(url: &str) -> Result<SkillSource> {
        let url = url
            .strip_prefix("https://github.com/")
            .or_else(|| url.strip_prefix("http://github.com/"))
            .ok_or_else(|| SkillxError::InvalidSource(format!("not a GitHub URL: {url}")))?;

        let parts: Vec<&str> = url.splitn(4, '/').collect();
        if parts.len() < 2 {
            return Err(SkillxError::InvalidSource(
                "invalid GitHub URL: cannot extract owner/repo".to_string(),
            ));
        }

        let owner = parts[0].to_string();
        let repo = parts[1].to_string();

        // Handle /tree/ref/path and /blob/ref/path
        let (path, ref_) = if parts.len() >= 4 && (parts[2] == "tree" || parts[2] == "blob") {
            let rest = parts[3];
            // ref is the first segment, path is the remainder
            if let Some((r, p)) = rest.split_once('/') {
                (Some(p.to_string()), Some(r.to_string()))
            } else {
                (None, Some(rest.to_string()))
            }
        } else {
            (None, None)
        };

        Ok(SkillSource::GitHub {
            owner,
            repo,
            path,
            ref_,
        })
    }

    /// Fetch a skill from GitHub using a three-tier strategy:
    ///   1. Archive tarball (no API rate limits)
    ///   2. Git clone (HTTPS first, SSH fallback)
    ///   3. Contents API with retry (last resort)
    pub async fn fetch(
        owner: &str,
        repo: &str,
        path: Option<&str>,
        ref_: Option<&str>,
        dest: &Path,
    ) -> Result<Vec<PathBuf>> {
        // Tier 1: Archive tarball
        let ref_name = ref_.unwrap_or("HEAD");
        let tarball_url =
            format!("https://github.com/{owner}/{repo}/archive/{ref_name}.tar.gz");
        let auth = std::env::var("GITHUB_TOKEN")
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
        let https_url = format!("https://github.com/{owner}/{repo}.git");
        let ssh_url = format!("git@github.com:{owner}/{repo}.git");

        if let Some(files) =
            super::git_clone::clone_skill(&https_url, Some(&ssh_url), path, ref_, dest).await
        {
            return Ok(files);
        }

        // Tier 3: Contents API with retry
        ui::info("Falling back to GitHub API...");
        Self::fetch_via_api(owner, repo, path, ref_, dest).await
    }

    /// Fetch via GitHub Contents API (fallback with retry support).
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

        let token = std::env::var("GITHUB_TOKEN").ok();
        let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(8));

        let ctx = ApiFetchContext {
            client: &client,
            token: &token,
            semaphore: &semaphore,
            owner,
            repo,
        };
        Self::fetch_dir_api(&ctx, path, ref_, dest).await
    }

    /// Recursively fetch a directory via Contents API with retry and concurrency limit.
    fn fetch_dir_api<'a>(
        ctx: &'a ApiFetchContext<'a>,
        path: Option<&'a str>,
        ref_: Option<&'a str>,
        dest: &'a Path,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<PathBuf>>> + Send + 'a>>
    {
        Box::pin(async move {
            let owner = ctx.owner;
            let repo = ctx.repo;
            let client = ctx.client;
            let token = ctx.token;
            let semaphore = ctx.semaphore;

            // Build the API URL
            let api_path = match path {
                Some(p) => super::urlencode_path(p),
                None => String::new(),
            };
            let mut url =
                format!("https://api.github.com/repos/{owner}/{repo}/contents/{api_path}");
            if let Some(r) = ref_ {
                let encoded_ref = super::urlencoding(r);
                url.push_str(&format!("?ref={encoded_ref}"));
            }

            // Use retry helper for the listing request
            let resp = super::git_clone::request_with_retry(
                || {
                    let mut req = client.get(&url);
                    if let Some(ref t) = token {
                        req = req.header("Authorization", format!("Bearer {t}"));
                    }
                    req
                },
                3,
            )
            .await?;

            match resp.status().as_u16() {
                401 => {
                    return Err(SkillxError::GitHubApi(
                        "authentication required. Set GITHUB_TOKEN environment variable.".into(),
                    ));
                }
                403 => {
                    return Err(SkillxError::GitHubApi(
                        "access denied. Repository may be private — set GITHUB_TOKEN.".into(),
                    ));
                }
                404 => {
                    return Err(SkillxError::GitHubApi(
                        "not found. Check the owner, repository, and path.".into(),
                    ));
                }
                s if !(200..300).contains(&s) => {
                    return Err(SkillxError::GitHubApi(format!(
                        "GitHub API returned HTTP {s}"
                    )));
                }
                _ => {}
            }

            let body: serde_json::Value = resp.json().await.map_err(|e| {
                SkillxError::GitHubApi(format!("failed to parse GitHub API response: {e}"))
            })?;

            std::fs::create_dir_all(dest)
                .map_err(|e| SkillxError::Source(format!("failed to create dir: {e}")))?;

            let mut downloaded_files = Vec::new();

            if let Some(arr) = body.as_array() {
                // Directory listing — download files with concurrency limit
                use futures::stream::{FuturesUnordered, StreamExt};

                let futures: FuturesUnordered<_> = arr
                    .iter()
                    .filter_map(|item| {
                        let name = item["name"].as_str()?;
                        let download_url = item["download_url"].as_str()?;
                        let file_type = item["type"].as_str()?;
                        if file_type != "file" {
                            return None;
                        }
                        let dest_path = dest.join(name);
                        let url = download_url.to_string();
                        let client = client.clone();
                        let token = token.clone();
                        let sem = semaphore.clone();
                        let name = name.to_string();
                        Some(async move {
                            let _permit = sem.acquire().await.map_err(|e| {
                                SkillxError::Network(format!("semaphore error: {e}"))
                            })?;
                            let resp = super::git_clone::request_with_retry(
                                || {
                                    let mut req = client.get(&url);
                                    if let Some(ref t) = token {
                                        req =
                                            req.header("Authorization", format!("Bearer {t}"));
                                    }
                                    req
                                },
                                3,
                            )
                            .await?;
                            if !resp.status().is_success() {
                                return Err(SkillxError::GitHubApi(format!(
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
                for r in results {
                    downloaded_files.push(r?);
                }

                // Recursively fetch subdirectories
                for item in arr {
                    if item["type"].as_str() == Some("dir") {
                        if let Some(name) = item["name"].as_str() {
                            let sub_path = match path {
                                Some(p) => format!("{p}/{name}"),
                                None => name.to_string(),
                            };
                            let sub_dest = dest.join(name);
                            let sub_files = Self::fetch_dir_api(
                                ctx,
                                Some(&sub_path),
                                ref_,
                                &sub_dest,
                            )
                            .await?;
                            downloaded_files.extend(sub_files);
                        }
                    }
                }
            } else if body["type"].as_str() == Some("file") {
                // Single file
                if let Some(content) = body["content"].as_str() {
                    let decoded = base64::Engine::decode(
                        &base64::engine::general_purpose::STANDARD,
                        content.replace('\n', ""),
                    )
                    .map_err(|e| SkillxError::GitHubApi(format!("base64 decode failed: {e}")))?;
                    let name = body["name"].as_str().unwrap_or("file");
                    let file_path = dest.join(name);
                    std::fs::write(&file_path, &decoded).map_err(|e| {
                        SkillxError::Source(format!(
                            "failed to write {}: {e}",
                            file_path.display()
                        ))
                    })?;
                    downloaded_files.push(file_path);
                }
            }

            Ok(downloaded_files)
        })
    }
}

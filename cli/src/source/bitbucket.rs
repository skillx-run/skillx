use std::path::{Path, PathBuf};

use crate::error::{Result, SkillxError};
use crate::source::SkillSource;

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

    /// Fetch a skill from Bitbucket using the Source API.
    ///
    /// API: GET /2.0/repositories/:owner/:repo/src/:ref/:path
    /// Directory listings return paginated JSON with `values[]` entries.
    pub async fn fetch(
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
    async fn fetch_dir(
        ctx: &FetchContext,
        path: &str,
        dest: &Path,
    ) -> Result<Vec<PathBuf>> {
        let url = format!(
            "https://api.bitbucket.org/2.0/repositories/{}/{}/src/{}/{path}",
            ctx.owner, ctx.repo, ctx.ref_,
        );

        let req = ctx.auth_request(ctx.client.get(&url));
        let resp = req.send().await.map_err(|e| {
            SkillxError::Network(format!("Bitbucket API request failed: {e}"))
        })?;

        if !resp.status().is_success() {
            return Err(SkillxError::BitbucketApi(format!(
                "Bitbucket API returned {} for path '{path}'",
                resp.status()
            )));
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| {
            SkillxError::BitbucketApi(format!("failed to parse Bitbucket response: {e}"))
        })?;

        let values = body["values"]
            .as_array()
            .ok_or_else(|| {
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
                    let mut req = client.get(&download_url);
                    if let Some(ref t) = token {
                        req = req.bearer_auth(t);
                    } else if let Some((ref user, ref pass)) = basic_auth {
                        req = req.basic_auth(user, Some(pass));
                    }
                    req = req.header("Accept", "application/octet-stream");
                    let resp = req.send().await.map_err(|e| {
                        SkillxError::Network(format!("download failed for {name}: {e}"))
                    })?;
                    if !resp.status().is_success() {
                        return Err(SkillxError::BitbucketApi(format!(
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
fn strip_root_prefix<'a>(file_path: &'a str, root_path: &str) -> &'a str {
    if !root_path.is_empty() {
        file_path
            .strip_prefix(root_path)
            .and_then(|p| p.strip_prefix('/'))
            .unwrap_or(file_path)
    } else {
        file_path
    }
}

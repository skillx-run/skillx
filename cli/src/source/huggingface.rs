use std::path::{Path, PathBuf};

use crate::error::{Result, SkillxError};
use crate::source::HfRepoType;

pub struct HuggingFaceSource;

impl HuggingFaceSource {
    /// Fetch a skill from HuggingFace using the REST API.
    ///
    /// - List directory: `GET https://huggingface.co/api/{type}/{owner}/{repo}/tree/{rev}/{path}`
    /// - Download file: `GET https://huggingface.co/{owner}/{repo}/resolve/{rev}/{filepath}`
    pub async fn fetch(
        owner: &str,
        repo: &str,
        path: Option<&str>,
        ref_: Option<&str>,
        repo_type: &HfRepoType,
        dest: &Path,
    ) -> Result<Vec<PathBuf>> {
        let ref_name = ref_.unwrap_or("main");
        let type_prefix = match repo_type {
            HfRepoType::Models => "models",
            HfRepoType::Datasets => "datasets",
            HfRepoType::Spaces => "spaces",
        };

        let client = reqwest::Client::builder()
            .user_agent("skillx/0.3")
            .build()
            .map_err(|e| SkillxError::Network(format!("failed to create HTTP client: {e}")))?;

        let token = std::env::var("HF_TOKEN").ok();

        std::fs::create_dir_all(dest)
            .map_err(|e| SkillxError::Source(format!("failed to create dest dir: {e}")))?;

        let path_suffix = path.unwrap_or("");
        let files = Self::fetch_recursive(
            &client,
            token.as_deref(),
            type_prefix,
            owner,
            repo,
            ref_name,
            path_suffix,
            dest,
            path_suffix,
        )
        .await?;

        Ok(files)
    }

    /// Recursively list and download files from a HuggingFace repo path.
    async fn fetch_recursive(
        client: &reqwest::Client,
        token: Option<&str>,
        type_prefix: &str,
        owner: &str,
        repo: &str,
        ref_name: &str,
        api_path: &str,
        dest: &Path,
        base_path: &str,
    ) -> Result<Vec<PathBuf>> {
        // Build list URL
        let list_url = if api_path.is_empty() {
            format!(
                "https://huggingface.co/api/{type_prefix}/{owner}/{repo}/tree/{ref_name}"
            )
        } else {
            format!(
                "https://huggingface.co/api/{type_prefix}/{owner}/{repo}/tree/{ref_name}/{api_path}"
            )
        };

        let mut req = client.get(&list_url);
        if let Some(t) = token {
            req = req.header("Authorization", format!("Bearer {t}"));
        }

        let resp = req.send().await.map_err(|e| {
            SkillxError::Network(format!("HuggingFace API request failed: {e}"))
        })?;

        if !resp.status().is_success() {
            return Err(SkillxError::HuggingFaceApi(format!(
                "HuggingFace API returned HTTP {} for {}",
                resp.status(),
                list_url
            )));
        }

        let entries: Vec<serde_json::Value> = resp.json().await.map_err(|e| {
            SkillxError::HuggingFaceApi(format!(
                "failed to parse HuggingFace API response: {e}"
            ))
        })?;

        let mut files = Vec::new();

        // Collect file download tasks
        use futures::stream::{FuturesUnordered, StreamExt};

        let futures: FuturesUnordered<_> = entries
            .iter()
            .filter_map(|entry| {
                let entry_type = entry["type"].as_str()?;
                let rfilename = entry["path"].as_str()?;

                if entry_type != "file" {
                    return None;
                }

                // Compute relative path from base
                let relative = if base_path.is_empty() {
                    rfilename.to_string()
                } else {
                    rfilename
                        .strip_prefix(base_path)
                        .unwrap_or(rfilename)
                        .trim_start_matches('/')
                        .to_string()
                };

                let dest_path = dest.join(&relative);
                let download_url = Self::resolve_download_url(
                    type_prefix, owner, repo, ref_name, rfilename,
                );
                let client = client.clone();
                let token = token.map(|t| t.to_string());

                Some(async move {
                    let mut req = client.get(&download_url);
                    if let Some(ref t) = token {
                        req = req.header("Authorization", format!("Bearer {t}"));
                    }
                    let resp = req.send().await.map_err(|e| {
                        SkillxError::Network(format!(
                            "HuggingFace download failed for {rfilename}: {e}"
                        ))
                    })?;
                    if !resp.status().is_success() {
                        return Err(SkillxError::HuggingFaceApi(format!(
                            "HuggingFace download returned HTTP {} for {rfilename}",
                            resp.status()
                        )));
                    }
                    let bytes = resp.bytes().await.map_err(|e| {
                        SkillxError::Network(format!(
                            "failed to read HuggingFace file {rfilename}: {e}"
                        ))
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
            files.push(r?);
        }

        // Recurse into directories
        for entry in &entries {
            if entry["type"].as_str() == Some("directory") {
                if let Some(dir_path) = entry["path"].as_str() {
                    let relative_dir = if base_path.is_empty() {
                        dir_path.to_string()
                    } else {
                        dir_path
                            .strip_prefix(base_path)
                            .unwrap_or(dir_path)
                            .trim_start_matches('/')
                            .to_string()
                    };
                    let sub_dest = dest.join(&relative_dir);
                    std::fs::create_dir_all(&sub_dest).map_err(|e| {
                        SkillxError::Source(format!("failed to create dir: {e}"))
                    })?;
                    let sub_files = Box::pin(Self::fetch_recursive(
                        client,
                        token,
                        type_prefix,
                        owner,
                        repo,
                        ref_name,
                        dir_path,
                        dest,
                        base_path,
                    ))
                    .await?;
                    files.extend(sub_files);
                }
            }
        }

        Ok(files)
    }

    /// Build the download URL for a file in a HuggingFace repo.
    ///
    /// For models: `https://huggingface.co/{owner}/{repo}/resolve/{ref}/{path}`
    /// For datasets: `https://huggingface.co/datasets/{owner}/{repo}/resolve/{ref}/{path}`
    /// For spaces: `https://huggingface.co/spaces/{owner}/{repo}/resolve/{ref}/{path}`
    fn resolve_download_url(
        type_prefix: &str,
        owner: &str,
        repo: &str,
        ref_name: &str,
        filepath: &str,
    ) -> String {
        if type_prefix == "models" {
            format!(
                "https://huggingface.co/{owner}/{repo}/resolve/{ref_name}/{filepath}"
            )
        } else {
            format!(
                "https://huggingface.co/{type_prefix}/{owner}/{repo}/resolve/{ref_name}/{filepath}"
            )
        }
    }
}

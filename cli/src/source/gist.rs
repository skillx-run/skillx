use std::path::{Path, PathBuf};

use crate::error::{Result, SkillxError};
use crate::source::SkillSource;

pub struct GistSource;

impl GistSource {
    /// Parse `gist:id[@revision]` prefix format.
    pub fn parse(input: &str) -> Result<SkillSource> {
        let input = input.trim();
        if input.is_empty() {
            return Err(SkillxError::InvalidSource(
                "invalid gist source: empty ID".into(),
            ));
        }

        let (id, revision) = if let Some((id, rev)) = input.split_once('@') {
            (id.to_string(), Some(rev.to_string()))
        } else {
            (input.to_string(), None)
        };

        Ok(SkillSource::Gist { id, revision })
    }

    /// Fetch all files from a GitHub Gist.
    ///
    /// API: GET /gists/:id returns JSON with `files` map containing content.
    pub async fn fetch(id: &str, revision: Option<&str>, dest: &Path) -> Result<Vec<PathBuf>> {
        let client = reqwest::Client::builder()
            .user_agent("skillx/0.3")
            .build()
            .map_err(|e| SkillxError::Network(format!("failed to create HTTP client: {e}")))?;

        let token = std::env::var("GITHUB_TOKEN").ok();

        let url = match revision {
            Some(rev) => format!("https://api.github.com/gists/{id}/{rev}"),
            None => format!("https://api.github.com/gists/{id}"),
        };

        let mut req = client.get(&url);
        if let Some(ref t) = token {
            req = req.header("Authorization", format!("Bearer {t}"));
        }

        let resp = req
            .send()
            .await
            .map_err(|e| SkillxError::Network(format!("Gist API request failed: {e}")))?;

        match resp.status().as_u16() {
            401 => {
                return Err(SkillxError::GistApi(
                    "authentication required. Set GITHUB_TOKEN environment variable.".into(),
                ));
            }
            403 => {
                return Err(SkillxError::GistApi(
                    "access denied. Gist may be private — set GITHUB_TOKEN.".into(),
                ));
            }
            404 => {
                return Err(SkillxError::GistApi(format!(
                    "gist '{id}' not found. Check the gist ID."
                )));
            }
            s if !(200..300).contains(&s) => {
                return Err(SkillxError::GistApi(format!("Gist API returned HTTP {s}")));
            }
            _ => {}
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| SkillxError::GistApi(format!("failed to parse Gist response: {e}")))?;

        let files = body["files"]
            .as_object()
            .ok_or_else(|| SkillxError::GistApi("unexpected Gist response: no files".into()))?;

        std::fs::create_dir_all(dest)
            .map_err(|e| SkillxError::Source(format!("failed to create dir: {e}")))?;

        let mut downloaded = Vec::new();
        for (filename, file_info) in files {
            if let Some(content) = file_info["content"].as_str() {
                let dest_path = dest.join(filename);
                std::fs::write(&dest_path, content).map_err(|e| {
                    SkillxError::Source(format!("failed to write {}: {e}", dest_path.display()))
                })?;
                downloaded.push(dest_path);
            } else if let Some(raw_url) = file_info["raw_url"].as_str() {
                // Large files don't have inline content; download via raw_url
                let mut req = client.get(raw_url);
                if let Some(ref t) = token {
                    req = req.header("Authorization", format!("Bearer {t}"));
                }
                let resp = req.send().await.map_err(|e| {
                    SkillxError::Network(format!("download failed for {filename}: {e}"))
                })?;
                if !resp.status().is_success() {
                    return Err(SkillxError::GistApi(format!(
                        "download failed for {filename}: HTTP {}",
                        resp.status()
                    )));
                }
                let bytes = resp
                    .bytes()
                    .await
                    .map_err(|e| SkillxError::Network(format!("failed to read {filename}: {e}")))?;
                let dest_path = dest.join(filename);
                std::fs::write(&dest_path, &bytes).map_err(|e| {
                    SkillxError::Source(format!("failed to write {}: {e}", dest_path.display()))
                })?;
                downloaded.push(dest_path);
            }
        }

        Ok(downloaded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_id() {
        let source = GistSource::parse("abc123").unwrap();
        match source {
            SkillSource::Gist { id, revision } => {
                assert_eq!(id, "abc123");
                assert!(revision.is_none());
            }
            _ => panic!("expected Gist source"),
        }
    }

    #[test]
    fn test_parse_with_revision() {
        let source = GistSource::parse("abc123@rev456").unwrap();
        match source {
            SkillSource::Gist { id, revision } => {
                assert_eq!(id, "abc123");
                assert_eq!(revision, Some("rev456".into()));
            }
            _ => panic!("expected Gist source"),
        }
    }

    #[test]
    fn test_parse_empty() {
        assert!(GistSource::parse("").is_err());
    }
}

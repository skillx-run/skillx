use crate::error::{Result, SkillxError};
use crate::source::url_patterns::{lookup_domain, UrlSourceType};
use crate::source::{ArchiveFormat, SkillSource, SkillsDirectoryPlatform};

/// Parse a URL into a `SkillSource`.
///
/// This function is **synchronous** — it only parses URL strings, never makes
/// network requests. Async operations (Gitea API probing, skills directory
/// page fetching) happen in the fetch phase.
///
/// Priority:
/// 1. github.com/owner/repo/tree/ref/path → GitHub
/// 2. gist.github.com/user/id → Gist
/// 3. gitlab.com/owner/repo/-/tree/ref/path → GitLab
/// 4. bitbucket.org/owner/repo/src/ref/path → Bitbucket
/// 5. codeberg.org/owner/repo/src/branch/ref/path → Gitea
/// 6. skills.sh / skillsmp.com / etc → SkillsDirectory
/// 7. URL ending in .zip / .tar.gz → Archive
/// 8. Unknown domain + /src/branch/ in path → Gitea (speculative)
/// 9. Fallback → Archive (attempt download, fail if not valid)
pub fn resolve_url(url: &str) -> Result<SkillSource> {
    let (host, path) = extract_host_path(url)?;

    // Check known domain patterns
    if let Some(source_type) = lookup_domain(&host) {
        return match source_type {
            UrlSourceType::GitHub => super::github::GitHubSource::parse_url(url),
            UrlSourceType::Gist => parse_gist_url(&path),
            UrlSourceType::GitLab => parse_gitlab_url(&host, &path),
            UrlSourceType::Bitbucket => parse_bitbucket_url(&path),
            UrlSourceType::Gitea => parse_gitea_url(&host, &path),
            UrlSourceType::SkillsDirectory => parse_skills_directory_url(&host, &path),
        };
    }

    // Archive detection by file extension
    if let Some(source) = try_parse_archive_url(url) {
        return Ok(source);
    }

    // Speculative Gitea detection: unknown domain + /src/branch/ pattern
    if path.contains("/src/branch/") || path.contains("/src/tag/") {
        return parse_gitea_url(&host, &path);
    }

    // Fallback: treat as archive download attempt
    Ok(SkillSource::Archive {
        url: url.to_string(),
        format: ArchiveFormat::Zip,
    })
}

/// Extract host and path from a URL string.
fn extract_host_path(url: &str) -> Result<(String, String)> {
    let without_scheme = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .ok_or_else(|| SkillxError::InvalidSource(format!("not a valid URL: {url}")))?;

    let (host, path) = match without_scheme.find('/') {
        Some(pos) => (
            without_scheme[..pos].to_lowercase(),
            without_scheme[pos..].to_string(),
        ),
        None => (without_scheme.to_lowercase(), "/".to_string()),
    };

    Ok((host, path))
}

/// Parse a gist.github.com URL: /user/id[/revision]
fn parse_gist_url(path: &str) -> Result<SkillSource> {
    let path = path.trim_start_matches('/').trim_end_matches('/');
    let parts: Vec<&str> = path.split('/').collect();

    // /user/id or just /id
    let (id, revision) = match parts.len() {
        0 => {
            return Err(SkillxError::InvalidSource(
                "invalid Gist URL: missing gist ID".into(),
            ))
        }
        1 => (parts[0].to_string(), None),
        2 => (parts[1].to_string(), None),
        _ => (parts[1].to_string(), Some(parts[2].to_string())),
    };

    if id.is_empty() {
        return Err(SkillxError::InvalidSource(
            "invalid Gist URL: empty gist ID".into(),
        ));
    }

    Ok(SkillSource::Gist { id, revision })
}

/// Parse a GitLab URL: /owner/repo/-/tree/ref/path
fn parse_gitlab_url(host: &str, path: &str) -> Result<SkillSource> {
    let path = path.trim_start_matches('/').trim_end_matches('/');
    let parts: Vec<&str> = path.splitn(3, '/').collect();

    if parts.len() < 2 {
        return Err(SkillxError::InvalidSource(format!(
            "invalid GitLab URL: cannot extract owner/repo from path '{path}'"
        )));
    }

    let owner = parts[0].to_string();
    let repo = parts[1].to_string();

    // Parse /-/tree/ref/path or /-/blob/ref/path
    let (ref_, sub_path) = if parts.len() >= 3 {
        let rest = parts[2];
        if let Some(tree_rest) = rest
            .strip_prefix("-/tree/")
            .or_else(|| rest.strip_prefix("-/blob/"))
        {
            if let Some((r, p)) = tree_rest.split_once('/') {
                (Some(r.to_string()), Some(p.to_string()))
            } else {
                (Some(tree_rest.to_string()), None)
            }
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    Ok(SkillSource::GitLab {
        host: host.to_string(),
        owner,
        repo,
        path: sub_path,
        ref_,
    })
}

/// Parse a Bitbucket URL: /owner/repo/src/ref/path
fn parse_bitbucket_url(path: &str) -> Result<SkillSource> {
    let path = path.trim_start_matches('/').trim_end_matches('/');
    let parts: Vec<&str> = path.splitn(3, '/').collect();

    if parts.len() < 2 {
        return Err(SkillxError::InvalidSource(format!(
            "invalid Bitbucket URL: cannot extract owner/repo from path '{path}'"
        )));
    }

    let owner = parts[0].to_string();
    let repo = parts[1].to_string();

    // Parse /src/ref/path
    let (ref_, sub_path) = if parts.len() >= 3 {
        let rest = parts[2];
        if let Some(src_rest) = rest.strip_prefix("src/") {
            if let Some((r, p)) = src_rest.split_once('/') {
                (Some(r.to_string()), Some(p.to_string()))
            } else {
                (Some(src_rest.to_string()), None)
            }
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    Ok(SkillSource::Bitbucket {
        owner,
        repo,
        path: sub_path,
        ref_,
    })
}

/// Parse a Gitea/Codeberg/Forgejo URL: /owner/repo/src/branch/ref/path
fn parse_gitea_url(host: &str, path: &str) -> Result<SkillSource> {
    let path = path.trim_start_matches('/').trim_end_matches('/');
    let parts: Vec<&str> = path.splitn(3, '/').collect();

    if parts.len() < 2 {
        return Err(SkillxError::InvalidSource(format!(
            "invalid Gitea URL: cannot extract owner/repo from path '{path}'"
        )));
    }

    let owner = parts[0].to_string();
    let repo = parts[1].to_string();

    // Parse /src/branch/ref/path or /src/tag/ref/path
    let (ref_, sub_path) = if parts.len() >= 3 {
        let rest = parts[2];
        if let Some(branch_rest) = rest
            .strip_prefix("src/branch/")
            .or_else(|| rest.strip_prefix("src/tag/"))
            .or_else(|| rest.strip_prefix("src/commit/"))
        {
            if let Some((r, p)) = branch_rest.split_once('/') {
                (Some(r.to_string()), Some(p.to_string()))
            } else {
                (Some(branch_rest.to_string()), None)
            }
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    Ok(SkillSource::Gitea {
        host: host.to_string(),
        owner,
        repo,
        path: sub_path,
        ref_,
    })
}

/// Parse a skills directory platform URL.
fn parse_skills_directory_url(host: &str, path: &str) -> Result<SkillSource> {
    let platform = match host {
        "skills.sh" => SkillsDirectoryPlatform::SkillsSh,
        "skillsmp.com" => SkillsDirectoryPlatform::SkillsMp,
        "clawhub.ai" => SkillsDirectoryPlatform::ClawHub,
        "lobehub.com" => SkillsDirectoryPlatform::LobeHub,
        "skillhub.club" => SkillsDirectoryPlatform::SkillHub,
        "agentskillshub.dev" => SkillsDirectoryPlatform::AgentSkillsHub,
        "agentskills.so" => SkillsDirectoryPlatform::AgentSkillsSo,
        "mcpmarket.com" => SkillsDirectoryPlatform::McpMarket,
        "skillsdirectory.com" => SkillsDirectoryPlatform::SkillsDirectory,
        "prompts.chat" => SkillsDirectoryPlatform::PromptsChat,
        _ => {
            return Err(SkillxError::UnsupportedUrl(format!(
                "unknown skills directory platform: {host}"
            )))
        }
    };

    Ok(SkillSource::SkillsDirectory {
        platform,
        path: path.to_string(),
    })
}

/// Try to parse a URL as an archive download (by file extension).
fn try_parse_archive_url(url: &str) -> Option<SkillSource> {
    let lower = url.to_lowercase();
    // Strip query string for extension check
    let path = lower.split('?').next().unwrap_or(&lower);

    if path.ends_with(".tar.gz") || path.ends_with(".tgz") {
        Some(SkillSource::Archive {
            url: url.to_string(),
            format: ArchiveFormat::TarGz,
        })
    } else if path.ends_with(".zip") {
        Some(SkillSource::Archive {
            url: url.to_string(),
            format: ArchiveFormat::Zip,
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_url() {
        let source = resolve_url("https://github.com/owner/repo/tree/main/path").unwrap();
        match source {
            SkillSource::GitHub {
                owner,
                repo,
                path,
                ref_,
            } => {
                assert_eq!(owner, "owner");
                assert_eq!(repo, "repo");
                assert_eq!(path, Some("path".into()));
                assert_eq!(ref_, Some("main".into()));
            }
            _ => panic!("expected GitHub source"),
        }
    }

    #[test]
    fn test_gist_url() {
        let source = resolve_url("https://gist.github.com/user/abc123").unwrap();
        match source {
            SkillSource::Gist { id, revision } => {
                assert_eq!(id, "abc123");
                assert!(revision.is_none());
            }
            _ => panic!("expected Gist source"),
        }
    }

    #[test]
    fn test_gist_url_with_revision() {
        let source = resolve_url("https://gist.github.com/user/abc123/rev456").unwrap();
        match source {
            SkillSource::Gist { id, revision } => {
                assert_eq!(id, "abc123");
                assert_eq!(revision, Some("rev456".into()));
            }
            _ => panic!("expected Gist source"),
        }
    }

    #[test]
    fn test_gitlab_url() {
        let source =
            resolve_url("https://gitlab.com/owner/repo/-/tree/main/skills/pdf").unwrap();
        match source {
            SkillSource::GitLab {
                host,
                owner,
                repo,
                path,
                ref_,
            } => {
                assert_eq!(host, "gitlab.com");
                assert_eq!(owner, "owner");
                assert_eq!(repo, "repo");
                assert_eq!(path, Some("skills/pdf".into()));
                assert_eq!(ref_, Some("main".into()));
            }
            _ => panic!("expected GitLab source"),
        }
    }

    #[test]
    fn test_gitlab_url_no_path() {
        let source = resolve_url("https://gitlab.com/owner/repo").unwrap();
        match source {
            SkillSource::GitLab {
                owner, repo, path, ..
            } => {
                assert_eq!(owner, "owner");
                assert_eq!(repo, "repo");
                assert!(path.is_none());
            }
            _ => panic!("expected GitLab source"),
        }
    }

    #[test]
    fn test_bitbucket_url() {
        let source =
            resolve_url("https://bitbucket.org/owner/repo/src/main/skills/pdf").unwrap();
        match source {
            SkillSource::Bitbucket {
                owner,
                repo,
                path,
                ref_,
            } => {
                assert_eq!(owner, "owner");
                assert_eq!(repo, "repo");
                assert_eq!(path, Some("skills/pdf".into()));
                assert_eq!(ref_, Some("main".into()));
            }
            _ => panic!("expected Bitbucket source"),
        }
    }

    #[test]
    fn test_codeberg_url() {
        let source =
            resolve_url("https://codeberg.org/owner/repo/src/branch/main/path").unwrap();
        match source {
            SkillSource::Gitea {
                host,
                owner,
                repo,
                path,
                ref_,
            } => {
                assert_eq!(host, "codeberg.org");
                assert_eq!(owner, "owner");
                assert_eq!(repo, "repo");
                assert_eq!(path, Some("path".into()));
                assert_eq!(ref_, Some("main".into()));
            }
            _ => panic!("expected Gitea source"),
        }
    }

    #[test]
    fn test_skills_directory_url() {
        let source = resolve_url("https://skills.sh/some/skill").unwrap();
        match source {
            SkillSource::SkillsDirectory { path, .. } => {
                assert_eq!(path, "/some/skill");
            }
            _ => panic!("expected SkillsDirectory source"),
        }
    }

    #[test]
    fn test_archive_zip_url() {
        let source = resolve_url("https://example.com/skill.zip").unwrap();
        match source {
            SkillSource::Archive { url, format } => {
                assert_eq!(url, "https://example.com/skill.zip");
                assert!(matches!(format, ArchiveFormat::Zip));
            }
            _ => panic!("expected Archive source"),
        }
    }

    #[test]
    fn test_archive_tar_gz_url() {
        let source = resolve_url("https://example.com/skill.tar.gz").unwrap();
        match source {
            SkillSource::Archive { url, format } => {
                assert_eq!(url, "https://example.com/skill.tar.gz");
                assert!(matches!(format, ArchiveFormat::TarGz));
            }
            _ => panic!("expected Archive source"),
        }
    }

    #[test]
    fn test_speculative_gitea() {
        let source =
            resolve_url("https://mygitea.example.com/owner/repo/src/branch/main/path")
                .unwrap();
        match source {
            SkillSource::Gitea {
                host, owner, repo, ..
            } => {
                assert_eq!(host, "mygitea.example.com");
                assert_eq!(owner, "owner");
                assert_eq!(repo, "repo");
            }
            _ => panic!("expected Gitea source"),
        }
    }

    #[test]
    fn test_unknown_url_fallback_archive() {
        let source = resolve_url("https://example.com/some/path").unwrap();
        assert!(matches!(source, SkillSource::Archive { .. }));
    }

    #[test]
    fn test_invalid_url() {
        assert!(resolve_url("not-a-url").is_err());
    }

    #[test]
    fn test_archive_zip_with_query() {
        let source = resolve_url("https://example.com/skill.zip?token=abc").unwrap();
        assert!(matches!(source, SkillSource::Archive { format: ArchiveFormat::Zip, .. }));
    }
}

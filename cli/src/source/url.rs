use crate::config::Config;
use crate::error::{Result, SkillxError};
use crate::source::url_patterns::{lookup_domain, lookup_domain_with_custom, UrlSourceType};
use crate::source::{ArchiveFormat, HfRepoType, SkillSource, SkillsDirectoryPlatform};

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
            UrlSourceType::SourceHut => parse_sourcehut_url(&path),
            UrlSourceType::HuggingFace => parse_huggingface_url(&path),
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
    let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    // Formats: /id, /user/id, /user/id/revision
    let (id, revision) = match parts.len() {
        0 => {
            return Err(SkillxError::InvalidSource(
                "invalid Gist URL: missing gist ID".into(),
            ))
        }
        1 => (parts[0].to_string(), None),
        2 => (parts[1].to_string(), None),
        3 => (parts[1].to_string(), Some(parts[2].to_string())),
        _ => {
            return Err(SkillxError::InvalidSource(
                "invalid Gist URL: too many path segments (expected /user/id[/revision])".into(),
            ))
        }
    };

    if id.is_empty() {
        return Err(SkillxError::InvalidSource(
            "invalid Gist URL: empty gist ID".into(),
        ));
    }

    Ok(SkillSource::Gist { id, revision })
}

/// Parse a GitLab URL: /owner/repo/-/tree/ref/path
///
/// GitLab uses `/-/` as a separator between namespace and actions.
/// Supports nested groups: `/group/subgroup/project/-/tree/main/path`
fn parse_gitlab_url(host: &str, path: &str) -> Result<SkillSource> {
    let path = path.trim_start_matches('/').trim_end_matches('/');

    // Split on /-/ to separate namespace/repo from tree/blob actions
    let (namespace_part, action_part) = if let Some(pos) = path.find("/-/") {
        (&path[..pos], Some(&path[pos + 3..]))
    } else {
        (path, None)
    };

    // namespace_part is "owner/repo" or "group/subgroup/project"
    // The last segment is the repo, everything before is the owner/group
    let segments: Vec<&str> = namespace_part.split('/').collect();
    if segments.len() < 2 {
        return Err(SkillxError::InvalidSource(format!(
            "invalid GitLab URL: cannot extract owner/repo from path '{path}'"
        )));
    }

    let repo = segments
        .last()
        .ok_or_else(|| {
            SkillxError::InvalidSource(format!(
                "invalid GitLab URL: empty namespace in path '{path}'"
            ))
        })?
        .to_string();
    let owner = segments[..segments.len() - 1].join("/");

    // Parse tree/ref/path or blob/ref/path from the action part
    let (ref_, sub_path) = if let Some(action) = action_part {
        if let Some(tree_rest) = action
            .strip_prefix("tree/")
            .or_else(|| action.strip_prefix("blob/"))
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

/// Resolve a URL using custom domain patterns from config before built-in patterns.
pub fn resolve_url_with_config(url: &str, config: &Config) -> Result<SkillSource> {
    let (host, path) = extract_host_path(url)?;

    // Check custom + built-in domain patterns
    if let Some(source_type) = lookup_domain_with_custom(&host, &config.url_patterns) {
        return match source_type {
            UrlSourceType::GitHub => super::github::GitHubSource::parse_url(url),
            UrlSourceType::Gist => parse_gist_url(&path),
            UrlSourceType::GitLab => parse_gitlab_url(&host, &path),
            UrlSourceType::Bitbucket => parse_bitbucket_url(&path),
            UrlSourceType::Gitea => parse_gitea_url(&host, &path),
            UrlSourceType::SourceHut => parse_sourcehut_url(&path),
            UrlSourceType::HuggingFace => parse_huggingface_url(&path),
            UrlSourceType::SkillsDirectory => parse_skills_directory_url(&host, &path),
        };
    }

    // Archive detection by file extension
    if let Some(source) = try_parse_archive_url(url) {
        return Ok(source);
    }

    // Speculative Gitea detection
    if path.contains("/src/branch/") || path.contains("/src/tag/") {
        return parse_gitea_url(&host, &path);
    }

    // Fallback: treat as archive download attempt
    Ok(SkillSource::Archive {
        url: url.to_string(),
        format: ArchiveFormat::Zip,
    })
}

/// Parse a SourceHut URL: /~owner/repo/tree/ref/item/path
///
/// SourceHut uses `~` prefix for owners and `item` keyword to separate ref and path.
fn parse_sourcehut_url(path: &str) -> Result<SkillSource> {
    let path = path.trim_start_matches('/').trim_end_matches('/');
    let parts: Vec<&str> = path.splitn(3, '/').collect();

    if parts.is_empty() {
        return Err(SkillxError::InvalidSource(
            "invalid SourceHut URL: missing owner/repo".into(),
        ));
    }

    // Strip ~ prefix from owner
    let owner = parts[0].strip_prefix('~').unwrap_or(parts[0]).to_string();
    if owner.is_empty() {
        return Err(SkillxError::InvalidSource(
            "invalid SourceHut URL: empty owner".into(),
        ));
    }

    if parts.len() < 2 {
        return Err(SkillxError::InvalidSource(
            "invalid SourceHut URL: missing repo name".into(),
        ));
    }

    let repo = parts[1].to_string();

    // Parse /tree/ref/item/path from the rest
    let (ref_, sub_path) = if parts.len() >= 3 {
        let rest = parts[2];
        if let Some(tree_rest) = rest.strip_prefix("tree/") {
            // Check for /item/ separator
            if let Some(item_pos) = tree_rest.find("/item/") {
                let ref_str = &tree_rest[..item_pos];
                let path_str = &tree_rest[item_pos + 6..]; // skip "/item/"
                (
                    Some(ref_str.to_string()),
                    if path_str.is_empty() {
                        None
                    } else {
                        Some(path_str.to_string())
                    },
                )
            } else {
                // Just ref, no path
                (Some(tree_rest.to_string()), None)
            }
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    Ok(SkillSource::SourceHut {
        owner,
        repo,
        path: sub_path,
        ref_,
    })
}

/// Parse a HuggingFace URL with repo type inference from path prefix.
///
/// Formats:
/// - `/owner/repo[/tree/ref[/path]]` → Models
/// - `/datasets/owner/repo[/tree/ref[/path]]` → Datasets
/// - `/spaces/owner/repo[/tree/ref[/path]]` → Spaces
fn parse_huggingface_url(path: &str) -> Result<SkillSource> {
    let path = path.trim_start_matches('/').trim_end_matches('/');
    let segments: Vec<&str> = path.split('/').collect();

    if segments.is_empty() {
        return Err(SkillxError::InvalidSource(
            "invalid HuggingFace URL: empty path".into(),
        ));
    }

    // Determine repo type and extract owner/repo
    let (repo_type, owner, repo, rest_start) = match segments[0] {
        "datasets" => {
            if segments.len() < 3 {
                return Err(SkillxError::InvalidSource(
                    "invalid HuggingFace URL: missing owner/repo after /datasets/".into(),
                ));
            }
            (HfRepoType::Datasets, segments[1], segments[2], 3)
        }
        "spaces" => {
            if segments.len() < 3 {
                return Err(SkillxError::InvalidSource(
                    "invalid HuggingFace URL: missing owner/repo after /spaces/".into(),
                ));
            }
            (HfRepoType::Spaces, segments[1], segments[2], 3)
        }
        _ => {
            if segments.len() < 2 {
                return Err(SkillxError::InvalidSource(
                    "invalid HuggingFace URL: missing owner/repo".into(),
                ));
            }
            (HfRepoType::Models, segments[0], segments[1], 2)
        }
    };

    // Parse /tree/ref/path from remaining segments
    let rest: Vec<&str> = segments[rest_start..].to_vec();
    let (ref_, sub_path) = if rest.len() >= 2 && rest[0] == "tree" {
        let ref_str = rest[1].to_string();
        let path_parts: Vec<&str> = rest[2..].to_vec();
        let path_str = if path_parts.is_empty() {
            None
        } else {
            Some(path_parts.join("/"))
        };
        (Some(ref_str), path_str)
    } else {
        (None, None)
    };

    Ok(SkillSource::HuggingFace {
        owner: owner.to_string(),
        repo: repo.to_string(),
        path: sub_path,
        ref_,
        repo_type,
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
        let source = resolve_url("https://gitlab.com/owner/repo/-/tree/main/skills/pdf").unwrap();
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
        let source = resolve_url("https://bitbucket.org/owner/repo/src/main/skills/pdf").unwrap();
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
        let source = resolve_url("https://codeberg.org/owner/repo/src/branch/main/path").unwrap();
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
            resolve_url("https://mygitea.example.com/owner/repo/src/branch/main/path").unwrap();
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
        assert!(matches!(
            source,
            SkillSource::Archive {
                format: ArchiveFormat::Zip,
                ..
            }
        ));
    }

    #[test]
    fn test_sourcehut_url_basic() {
        let source = resolve_url("https://git.sr.ht/~sircmpwn/hare/tree/master/item/cmd").unwrap();
        match source {
            SkillSource::SourceHut {
                owner,
                repo,
                path,
                ref_,
            } => {
                assert_eq!(owner, "sircmpwn");
                assert_eq!(repo, "hare");
                assert_eq!(path, Some("cmd".into()));
                assert_eq!(ref_, Some("master".into()));
            }
            _ => panic!("expected SourceHut source"),
        }
    }

    #[test]
    fn test_sourcehut_url_no_path() {
        let source = resolve_url("https://git.sr.ht/~owner/repo").unwrap();
        match source {
            SkillSource::SourceHut {
                owner,
                repo,
                path,
                ref_,
            } => {
                assert_eq!(owner, "owner");
                assert_eq!(repo, "repo");
                assert!(path.is_none());
                assert!(ref_.is_none());
            }
            _ => panic!("expected SourceHut source"),
        }
    }

    #[test]
    fn test_sourcehut_url_no_ref() {
        let source = resolve_url("https://git.sr.ht/~owner/repo/tree/main").unwrap();
        match source {
            SkillSource::SourceHut {
                owner,
                repo,
                path,
                ref_,
            } => {
                assert_eq!(owner, "owner");
                assert_eq!(repo, "repo");
                assert!(path.is_none());
                assert_eq!(ref_, Some("main".into()));
            }
            _ => panic!("expected SourceHut source"),
        }
    }

    #[test]
    fn test_huggingface_url_basic() {
        let source = resolve_url("https://huggingface.co/org/repo/tree/main/skills/pdf").unwrap();
        match source {
            SkillSource::HuggingFace {
                owner,
                repo,
                path,
                ref_,
                repo_type,
            } => {
                assert_eq!(owner, "org");
                assert_eq!(repo, "repo");
                assert_eq!(path, Some("skills/pdf".into()));
                assert_eq!(ref_, Some("main".into()));
                assert_eq!(repo_type, HfRepoType::Models);
            }
            _ => panic!("expected HuggingFace source"),
        }
    }

    #[test]
    fn test_huggingface_url_no_path() {
        let source = resolve_url("https://huggingface.co/org/repo").unwrap();
        match source {
            SkillSource::HuggingFace {
                owner,
                repo,
                path,
                ref_,
                repo_type,
            } => {
                assert_eq!(owner, "org");
                assert_eq!(repo, "repo");
                assert!(path.is_none());
                assert!(ref_.is_none());
                assert_eq!(repo_type, HfRepoType::Models);
            }
            _ => panic!("expected HuggingFace source"),
        }
    }

    #[test]
    fn test_huggingface_url_dataset() {
        let source = resolve_url("https://huggingface.co/datasets/org/repo").unwrap();
        match source {
            SkillSource::HuggingFace {
                repo_type,
                owner,
                repo,
                ..
            } => {
                assert_eq!(repo_type, HfRepoType::Datasets);
                assert_eq!(owner, "org");
                assert_eq!(repo, "repo");
            }
            _ => panic!("expected HuggingFace source"),
        }
    }

    #[test]
    fn test_huggingface_url_space() {
        let source = resolve_url("https://huggingface.co/spaces/org/repo").unwrap();
        match source {
            SkillSource::HuggingFace { repo_type, .. } => {
                assert_eq!(repo_type, HfRepoType::Spaces);
            }
            _ => panic!("expected HuggingFace source"),
        }
    }

    #[test]
    fn test_resolve_url_with_config_custom_gitea() {
        let config = Config {
            url_patterns: vec![crate::config::CustomUrlPattern {
                domain: "mygitea.company.com".to_string(),
                source_type: "gitea".to_string(),
            }],
            ..Config::default()
        };
        let source =
            resolve_url_with_config("https://mygitea.company.com/owner/repo", &config).unwrap();
        match source {
            SkillSource::Gitea {
                host, owner, repo, ..
            } => {
                assert_eq!(host, "mygitea.company.com");
                assert_eq!(owner, "owner");
                assert_eq!(repo, "repo");
            }
            _ => panic!("expected Gitea source"),
        }
    }
}

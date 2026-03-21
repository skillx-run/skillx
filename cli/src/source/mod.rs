pub mod archive;
pub mod bitbucket;
pub mod gist;
pub mod gitea;
pub mod github;
pub mod gitlab;
pub mod huggingface;
pub mod local;
pub mod resolver;
pub mod skills_directory;
pub mod sourcehut;
pub mod url;
pub mod url_patterns;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::error::{Result, SkillxError};

/// Percent-encode a string for use in URL query parameters.
pub fn urlencoding(s: &str) -> String {
    let mut encoded = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(b as char);
            }
            _ => {
                encoded.push_str(&format!("%{b:02X}"));
            }
        }
    }
    encoded
}

/// Percent-encode each segment of a URL path (split by `/`).
pub fn urlencode_path(s: &str) -> String {
    s.split('/')
        .map(|seg| urlencoding(seg))
        .collect::<Vec<_>>()
        .join("/")
}

/// Archive format for downloaded skill packages.
#[derive(Debug, Clone)]
pub enum ArchiveFormat {
    Zip,
    TarGz,
}

/// Skills directory platform identifier.
#[derive(Debug, Clone)]
pub enum SkillsDirectoryPlatform {
    SkillsSh,
    SkillsMp,
    ClawHub,
    LobeHub,
    SkillHub,
    AgentSkillsHub,
    AgentSkillsSo,
    McpMarket,
    SkillsDirectory,
    PromptsChat,
}

/// HuggingFace repository type.
#[derive(Debug, Clone, PartialEq)]
pub enum HfRepoType {
    Models,
    Datasets,
    Spaces,
}

/// Represents where a skill comes from.
#[derive(Debug, Clone)]
pub enum SkillSource {
    Local(PathBuf),
    GitHub {
        owner: String,
        repo: String,
        path: Option<String>,
        ref_: Option<String>,
    },
    GitLab {
        host: String,
        owner: String,
        repo: String,
        path: Option<String>,
        ref_: Option<String>,
    },
    Bitbucket {
        owner: String,
        repo: String,
        path: Option<String>,
        ref_: Option<String>,
    },
    Gitea {
        host: String,
        owner: String,
        repo: String,
        path: Option<String>,
        ref_: Option<String>,
    },
    Gist {
        id: String,
        revision: Option<String>,
    },
    Archive {
        url: String,
        format: ArchiveFormat,
    },
    SourceHut {
        owner: String,
        repo: String,
        path: Option<String>,
        ref_: Option<String>,
    },
    HuggingFace {
        owner: String,
        repo: String,
        path: Option<String>,
        ref_: Option<String>,
        repo_type: HfRepoType,
    },
    SkillsDirectory {
        platform: SkillsDirectoryPlatform,
        path: String,
    },
}

/// A resolved skill ready for scanning/injection.
#[derive(Debug, Clone)]
pub struct ResolvedSkill {
    pub source: SkillSource,
    pub metadata: SkillMetadata,
    pub root_dir: PathBuf,
    pub files: Vec<PathBuf>,
}

/// Skill metadata parsed from SKILL.md frontmatter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillMetadata {
    pub name: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub version: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// Resolve a source string into a `SkillSource`.
///
/// Priority: local path > explicit prefix > URL > bare name (error)
pub fn resolve(input: &str) -> Result<SkillSource> {
    let input = input.trim();

    // 1. Local path: starts with ./ or / or ~ or is an existing path
    if input.starts_with("./")
        || input.starts_with('/')
        || input.starts_with("~/")
        || input.starts_with(".\\")
    {
        let path = if let Some(rest) = input.strip_prefix("~/") {
            dirs::home_dir()
                .ok_or_else(|| SkillxError::Source("cannot determine home directory".into()))?
                .join(rest)
        } else {
            PathBuf::from(input)
        };
        return Ok(SkillSource::Local(path));
    }

    // Check if it's a relative path that exists on disk
    let as_path = PathBuf::from(input);
    if as_path.exists() {
        return Ok(SkillSource::Local(as_path));
    }

    // 2. Explicit prefixes: github: and gist:
    if let Some(rest) = input.strip_prefix("github:") {
        return github::GitHubSource::parse(rest);
    }
    if let Some(rest) = input.strip_prefix("gist:") {
        return gist::GistSource::parse(rest);
    }

    // 3. Full URL — URL smart recognition engine
    if input.starts_with("https://") || input.starts_with("http://") {
        return url::resolve_url(input);
    }

    // 4. Bare name — reserved for v0.4 registry
    Err(SkillxError::InvalidSource(format!(
        "cannot resolve source: '{input}'. Use a local path (./skill), github:/gist: prefix, or a full URL"
    )))
}

/// Resolve a source string into a `SkillSource`, using custom URL patterns from config.
pub fn resolve_with_config(input: &str, config: &crate::config::Config) -> Result<SkillSource> {
    let input = input.trim();

    // 1. Local path
    if input.starts_with("./")
        || input.starts_with('/')
        || input.starts_with("~/")
        || input.starts_with(".\\")
    {
        let path = if let Some(rest) = input.strip_prefix("~/") {
            dirs::home_dir()
                .ok_or_else(|| SkillxError::Source("cannot determine home directory".into()))?
                .join(rest)
        } else {
            std::path::PathBuf::from(input)
        };
        return Ok(SkillSource::Local(path));
    }

    let as_path = std::path::PathBuf::from(input);
    if as_path.exists() {
        return Ok(SkillSource::Local(as_path));
    }

    // 2. Explicit prefixes
    if let Some(rest) = input.strip_prefix("github:") {
        return github::GitHubSource::parse(rest);
    }
    if let Some(rest) = input.strip_prefix("gist:") {
        return gist::GistSource::parse(rest);
    }

    // 3. Full URL — with custom patterns
    if input.starts_with("https://") || input.starts_with("http://") {
        return url::resolve_url_with_config(input, config);
    }

    Err(SkillxError::InvalidSource(format!(
        "cannot resolve source: '{input}'. Use a local path (./skill), github:/gist: prefix, or a full URL"
    )))
}

/// Parse YAML frontmatter from SKILL.md content.
///
/// Frontmatter is delimited by `---` lines at the start of the file.
pub fn parse_frontmatter(content: &str) -> Result<SkillMetadata> {
    let content = content.trim_start();
    let after_first = match content.strip_prefix("---") {
        Some(rest) => rest,
        None => return Ok(SkillMetadata::default()),
    };

    let end = after_first
        .find("\n---")
        .ok_or_else(|| SkillxError::FrontmatterParse("unclosed frontmatter block".into()))?;

    let yaml = &after_first[..end];
    let metadata: SkillMetadata = serde_yaml::from_str(yaml)
        .map_err(|e| SkillxError::FrontmatterParse(format!("invalid YAML: {e}")))?;

    Ok(metadata)
}

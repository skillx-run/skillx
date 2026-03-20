pub mod github;
pub mod local;

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
/// Priority: local path > `github:` prefix > GitHub URL > error
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

    // 2. github: prefix
    if let Some(rest) = input.strip_prefix("github:") {
        return github::GitHubSource::parse(rest);
    }

    // 3. GitHub URL
    if input.starts_with("https://github.com/") || input.starts_with("http://github.com/") {
        return github::GitHubSource::parse_url(input);
    }

    // 4. Bare name — not supported in v0.1
    Err(SkillxError::InvalidSource(format!(
        "cannot resolve source: '{input}'. Use a local path (./skill), github: prefix, or GitHub URL"
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

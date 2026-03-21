use std::path::PathBuf;

use crate::source::local::LocalSource;
use crate::source::{self, SkillSource};
use crate::cache::CacheManager;
use crate::config::Config;
use crate::ui;

/// A resolved skill with its local directory and name.
pub struct FetchedSkill {
    pub dir: PathBuf,
    pub name: String,
}

/// Resolve a source string and fetch skill files to a local directory.
///
/// Integrates: resolve → cache check → fetch → return FetchedSkill.
pub async fn resolve_and_fetch(
    input: &str,
    no_cache: bool,
    config: &Config,
) -> anyhow::Result<FetchedSkill> {
    let skill_source = source::resolve_with_config(input, config)?;

    match skill_source {
        SkillSource::Local(path) => {
            let resolved = LocalSource::fetch(&path)?;
            let name = resolved
                .metadata
                .name
                .clone()
                .unwrap_or_else(|| {
                    path.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or("skill".into())
                });
            Ok(FetchedSkill {
                dir: resolved.root_dir,
                name,
            })
        }
        SkillSource::GitHub {
            owner,
            repo,
            path,
            ref_,
        } => {
            let cache_key = input.to_string();
            let dir = fetch_with_cache(&cache_key, no_cache, || async {
                let dest = cache_dest(&cache_key)?;
                source::github::GitHubSource::fetch(
                    &owner,
                    &repo,
                    path.as_deref(),
                    ref_.as_deref(),
                    &dest,
                )
                .await?;
                Ok(dest)
            })
            .await?;
            let resolved = LocalSource::fetch(&dir)?;
            let name = resolved
                .metadata
                .name
                .clone()
                .unwrap_or_else(|| path.as_deref().unwrap_or(&repo).to_string());
            Ok(FetchedSkill { dir, name })
        }
        SkillSource::GitLab {
            host,
            owner,
            repo,
            path,
            ref_,
        } => {
            let cache_key = input.to_string();
            let dir = fetch_with_cache(&cache_key, no_cache, || async {
                let dest = cache_dest(&cache_key)?;
                source::gitlab::GitLabSource::fetch(
                    &host,
                    &owner,
                    &repo,
                    path.as_deref(),
                    ref_.as_deref(),
                    &dest,
                )
                .await?;
                Ok(dest)
            })
            .await?;
            let resolved = LocalSource::fetch(&dir)?;
            let name = resolved
                .metadata
                .name
                .clone()
                .unwrap_or_else(|| path.as_deref().unwrap_or(&repo).to_string());
            Ok(FetchedSkill { dir, name })
        }
        SkillSource::Bitbucket {
            owner,
            repo,
            path,
            ref_,
        } => {
            let cache_key = input.to_string();
            let dir = fetch_with_cache(&cache_key, no_cache, || async {
                let dest = cache_dest(&cache_key)?;
                source::bitbucket::BitbucketSource::fetch(
                    &owner,
                    &repo,
                    path.as_deref(),
                    ref_.as_deref(),
                    &dest,
                )
                .await?;
                Ok(dest)
            })
            .await?;
            let resolved = LocalSource::fetch(&dir)?;
            let name = resolved
                .metadata
                .name
                .clone()
                .unwrap_or_else(|| path.as_deref().unwrap_or(&repo).to_string());
            Ok(FetchedSkill { dir, name })
        }
        SkillSource::Gitea {
            host,
            owner,
            repo,
            path,
            ref_,
        } => {
            let cache_key = input.to_string();
            let dir = fetch_with_cache(&cache_key, no_cache, || async {
                let dest = cache_dest(&cache_key)?;
                source::gitea::GiteaSource::fetch(
                    &host,
                    &owner,
                    &repo,
                    path.as_deref(),
                    ref_.as_deref(),
                    &dest,
                )
                .await?;
                Ok(dest)
            })
            .await?;
            let resolved = LocalSource::fetch(&dir)?;
            let name = resolved
                .metadata
                .name
                .clone()
                .unwrap_or_else(|| path.as_deref().unwrap_or(&repo).to_string());
            Ok(FetchedSkill { dir, name })
        }
        SkillSource::Gist { id, revision } => {
            let cache_key = input.to_string();
            let dir = fetch_with_cache(&cache_key, no_cache, || async {
                let dest = cache_dest(&cache_key)?;
                source::gist::GistSource::fetch(
                    &id,
                    revision.as_deref(),
                    &dest,
                )
                .await?;
                Ok(dest)
            })
            .await?;
            let resolved = LocalSource::fetch(&dir)?;
            let name = resolved
                .metadata
                .name
                .clone()
                .unwrap_or_else(|| format!("gist-{}", &id[..8.min(id.len())]));
            Ok(FetchedSkill { dir, name })
        }
        SkillSource::SourceHut {
            owner,
            repo,
            path,
            ref_,
        } => {
            let cache_key = input.to_string();
            let dir = fetch_with_cache(&cache_key, no_cache, || async {
                let dest = cache_dest(&cache_key)?;
                source::sourcehut::SourceHutSource::fetch(
                    &owner,
                    &repo,
                    path.as_deref(),
                    ref_.as_deref(),
                    &dest,
                )
                .await?;
                Ok(dest)
            })
            .await?;
            let resolved = LocalSource::fetch(&dir)?;
            let name = resolved
                .metadata
                .name
                .clone()
                .unwrap_or_else(|| path.as_deref().unwrap_or(&repo).to_string());
            Ok(FetchedSkill { dir, name })
        }
        SkillSource::HuggingFace {
            owner,
            repo,
            path,
            ref_,
            repo_type,
        } => {
            let cache_key = input.to_string();
            let dir = fetch_with_cache(&cache_key, no_cache, || async {
                let dest = cache_dest(&cache_key)?;
                source::huggingface::HuggingFaceSource::fetch(
                    &owner,
                    &repo,
                    path.as_deref(),
                    ref_.as_deref(),
                    &repo_type,
                    &dest,
                )
                .await?;
                Ok(dest)
            })
            .await?;
            let resolved = LocalSource::fetch(&dir)?;
            let name = resolved
                .metadata
                .name
                .clone()
                .unwrap_or_else(|| path.as_deref().unwrap_or(&repo).to_string());
            Ok(FetchedSkill { dir, name })
        }
        SkillSource::Archive { url, format } => {
            // Gitea API probe: for fallback Archive URLs (no archive extension),
            // try detecting if the host is a Gitea instance
            let is_fallback = {
                let lower = url.to_lowercase();
                let path_part = lower.split('?').next().unwrap_or(&lower);
                !path_part.ends_with(".zip")
                    && !path_part.ends_with(".tar.gz")
                    && !path_part.ends_with(".tgz")
            };

            if is_fallback {
                if let Some(gitea_result) =
                    try_gitea_probe(&url, no_cache, config).await
                {
                    return gitea_result;
                }
            }

            let cache_key = input.to_string();
            let dir = fetch_with_cache(&cache_key, no_cache, || async {
                let dest = cache_dest(&cache_key)?;
                source::archive::ArchiveSource::fetch(&url, &format, &dest).await?;
                Ok(dest)
            })
            .await?;
            let resolved = LocalSource::fetch(&dir)?;
            let name = resolved
                .metadata
                .name
                .clone()
                .unwrap_or_else(|| "archive-skill".into());
            Ok(FetchedSkill { dir, name })
        }
        SkillSource::SkillsDirectory { platform, path } => {
            // Resolve to underlying GitHub source, then fetch
            ui::step("Resolving skills directory source...");
            let resolved_source =
                source::skills_directory::resolve_skills_directory(&platform, &path).await?;
            match resolved_source {
                SkillSource::GitHub {
                    owner, repo, path, ref_,
                } => {
                    let mut source_url = format!("https://github.com/{owner}/{repo}");
                    if let Some(r) = &ref_ {
                        source_url.push_str(&format!("/tree/{r}"));
                        if let Some(p) = &path {
                            source_url.push_str(&format!("/{p}"));
                        }
                    }
                    Box::pin(resolve_and_fetch(&source_url, no_cache, config)).await
                }
                _ => Err(anyhow::anyhow!(
                    "skills directory resolved to unsupported source type"
                )),
            }
        }
    }
}

/// Fetch with cache: check cache first, then fetch if miss.
async fn fetch_with_cache<F, Fut>(
    cache_key: &str,
    no_cache: bool,
    fetch_fn: F,
) -> anyhow::Result<PathBuf>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = anyhow::Result<PathBuf>>,
{
    if !no_cache {
        if let Some(cached) = CacheManager::lookup(cache_key)? {
            ui::success("Using cached copy");
            return Ok(cached);
        }
    }

    Config::ensure_dirs()?;
    let sp = ui::spinner("Fetching from remote...");
    let dest = fetch_fn().await?;
    sp.finish_and_clear();
    ui::success("Downloaded successfully");
    Ok(dest)
}

/// Get the cache destination directory for a given key.
fn cache_dest(cache_key: &str) -> anyhow::Result<PathBuf> {
    Ok(Config::cache_dir()?
        .join(CacheManager::source_hash(cache_key))
        .join("skill-files"))
}

/// Try to detect if a fallback Archive URL is actually a Gitea instance.
///
/// Probes `GET {scheme}://{host}/api/v1/settings/api` with a 2-second timeout.
/// If successful, parses owner/repo from the URL path and fetches via Gitea.
async fn try_gitea_probe(
    url: &str,
    no_cache: bool,
    config: &Config,
) -> Option<anyhow::Result<FetchedSkill>> {
    // Extract scheme and host from URL
    let (scheme, without_scheme) = if let Some(rest) = url.strip_prefix("https://") {
        ("https", rest)
    } else if let Some(rest) = url.strip_prefix("http://") {
        ("http", rest)
    } else {
        return None;
    };

    let slash_pos = without_scheme.find('/');
    let host = match slash_pos {
        Some(pos) => &without_scheme[..pos],
        None => without_scheme,
    };

    // Probe Gitea API using the same scheme as the original URL
    let probe_url = format!("{scheme}://{host}/api/v1/settings/api");
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .ok()?;

    let resp = client.get(&probe_url).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }

    // Parse owner/repo from URL path (first two segments)
    let path = match slash_pos {
        Some(pos) => &without_scheme[pos + 1..],
        None => return None,
    };
    let path = path.trim_end_matches('/');
    let segments: Vec<&str> = path.split('/').collect();
    if segments.len() < 2 {
        return None;
    }

    let owner = segments[0];
    let repo = segments[1];

    ui::info(&format!("Detected Gitea instance at {host}"));

    // Re-resolve as Gitea URL using the original scheme
    let gitea_url = format!("{scheme}://{host}/{owner}/{repo}");
    Some(Box::pin(resolve_and_fetch(&gitea_url, no_cache, config)).await)
}

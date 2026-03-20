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
) -> anyhow::Result<FetchedSkill> {
    let skill_source = source::resolve(input)?;

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
        SkillSource::Archive { url, format } => {
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
                    Box::pin(resolve_and_fetch(&source_url, no_cache)).await
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

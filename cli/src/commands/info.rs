use clap::Args;

use skillx::source;
use skillx::source::local::LocalSource;
use skillx::ui;

#[derive(Args, Debug)]
pub struct InfoArgs {
    /// Skill source to inspect
    pub source: String,
}

pub async fn execute(args: InfoArgs) -> anyhow::Result<()> {
    ui::step("Resolving source...");
    let skill_source = source::resolve(&args.source)?;

    let (metadata, files, root_dir) = match &skill_source {
        source::SkillSource::Local(path) => {
            let resolved = LocalSource::fetch(path)?;
            (resolved.metadata, resolved.files, resolved.root_dir)
        }
        source::SkillSource::GitHub {
            owner,
            repo,
            path,
            ref_,
        } => {
            let cache_key = args.source.clone();

            // Check cache
            let skill_dir = if let Some(cached) = skillx::cache::CacheManager::lookup(&cache_key)? {
                ui::success("Using cached copy");
                cached
            } else {
                let sp = ui::spinner("Fetching from GitHub...");
                skillx::config::Config::ensure_dirs()?;
                let dest = skillx::config::Config::cache_dir()?
                    .join(skillx::cache::CacheManager::source_hash(&cache_key))
                    .join("skill-files");
                source::github::GitHubSource::fetch(
                    owner,
                    repo,
                    path.as_deref(),
                    ref_.as_deref(),
                    &dest,
                )
                .await?;
                sp.finish_and_clear();
                dest
            };

            let resolved = LocalSource::fetch(&skill_dir)?;
            (resolved.metadata, resolved.files, resolved.root_dir)
        }
    };

    // Display info
    ui::header("Skill Information");
    ui::kv("Name", metadata.name.as_deref().unwrap_or("(unnamed)"));
    ui::kv(
        "Description",
        metadata.description.as_deref().unwrap_or("(none)"),
    );
    ui::kv("Author", metadata.author.as_deref().unwrap_or("(unknown)"));
    ui::kv("Version", metadata.version.as_deref().unwrap_or("(none)"));

    if let Some(ref tags) = metadata.tags {
        ui::kv("Tags", &tags.join(", "));
    }

    ui::kv("Source", &args.source);
    ui::kv("Path", &root_dir.display().to_string());

    eprintln!();
    ui::header("Files");
    for file in &files {
        let rel = file
            .strip_prefix(&root_dir)
            .unwrap_or(file);
        eprintln!("  {}", rel.display());
    }
    eprintln!("  ({} files total)", files.len());

    Ok(())
}

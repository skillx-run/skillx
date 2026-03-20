use clap::Args;

use skillx::source::local::LocalSource;
use skillx::source::resolver;
use skillx::ui;

#[derive(Args, Debug)]
pub struct InfoArgs {
    /// Skill source to inspect
    pub source: String,
}

pub async fn execute(args: InfoArgs) -> anyhow::Result<()> {
    ui::step("Resolving source...");
    let fetched = resolver::resolve_and_fetch(&args.source, false).await?;

    // Re-fetch metadata from the resolved directory
    let resolved = LocalSource::fetch(&fetched.dir)?;
    let metadata = resolved.metadata;
    let files = resolved.files;
    let root_dir = resolved.root_dir;

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

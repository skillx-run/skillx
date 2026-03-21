use clap::Args;
use std::path::Path;

use skillx::installed::InstalledState;
use skillx::project_config::ProjectConfig;
use skillx::ui;

#[derive(Args, Debug)]
pub struct InitArgs {
    /// Generate skillx.toml from currently installed skills
    #[arg(long)]
    pub from_installed: bool,
}

pub async fn execute(args: InitArgs) -> anyhow::Result<()> {
    let path = Path::new("skillx.toml");
    if path.exists() {
        return Err(anyhow::anyhow!(
            "skillx.toml already exists in the current directory"
        ));
    }

    if args.from_installed {
        let installed = match InstalledState::load() {
            Ok(state) => state,
            Err(e) => {
                ui::warn(&format!("Failed to load installed state: {e}"));
                ui::warn("Creating empty skillx.toml instead.");
                InstalledState::default()
            }
        };
        if installed.skills.is_empty() {
            ui::warn("No skills installed. Creating empty skillx.toml.");
            ProjectConfig::create_default(Path::new("."))?;
        } else {
            let skills: Vec<(String, String)> = installed
                .skills
                .iter()
                .map(|s| (s.name.clone(), s.source.clone()))
                .collect();
            ProjectConfig::create_from_installed(Path::new("."), &skills)?;
            ui::success(&format!(
                "Created skillx.toml with {} skill(s) from installed state",
                skills.len()
            ));
        }
    } else {
        ProjectConfig::create_default(Path::new("."))?;
        ui::success("Created skillx.toml");
    }

    ui::info("Next steps:");
    ui::info("  1. Edit skillx.toml to add skills");
    ui::info("  2. Run 'skillx install' to install from skillx.toml");

    Ok(())
}

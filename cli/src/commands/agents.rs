use clap::Args;
use console::style;

use skillx::agent::registry::AgentRegistry;
use skillx::agent::LifecycleMode;
use skillx::config::Config;
use skillx::types::Scope;
use skillx::ui;

#[derive(Args, Debug)]
pub struct AgentsArgs {
    /// Show all known agents (including undetected)
    #[arg(long)]
    pub all: bool,
}

pub async fn execute(args: AgentsArgs) -> anyhow::Result<()> {
    let config = Config::load()?;
    let registry = AgentRegistry::new(&config);
    let results = registry.detect_all().await;

    let total = results.len();
    let detected_count = results.iter().filter(|r| r.detected).count();

    eprintln!(
        "\n{}",
        style(format!(
            "Detected Agent environments ({detected_count} of {total} supported):"
        ))
        .bold()
        .underlined()
    );
    eprintln!();

    let mut found_any = false;

    for result in &results {
        if !args.all && !result.detected {
            continue;
        }

        found_any = true;

        let adapter = registry.get(&result.name);

        let status_icon = if result.detected {
            style("✓").green().to_string()
        } else {
            style("✗").dim().to_string()
        };

        let name_display = format!("{:<16}", result.name);

        let version_display = if result.detected {
            result
                .version
                .as_ref()
                .map(|v| format!("v{v}"))
                .unwrap_or_else(|| "—".to_string())
        } else {
            "—".to_string()
        };

        if result.detected {
            let inject_path = adapter
                .map(|a| {
                    let p = a.inject_path("<skill>", &Scope::Project);
                    p.display().to_string()
                })
                .unwrap_or_default();

            let mode_label = adapter
                .map(|a| match a.lifecycle_mode() {
                    LifecycleMode::ManagedProcess => "CLI, managed-process",
                    LifecycleMode::FileInjectAndWait => "IDE, file-inject",
                })
                .unwrap_or("unknown");

            eprintln!(
                "  {} {}  {:<12} {:<28} ({})",
                status_icon, name_display, version_display, inject_path, mode_label
            );
        } else {
            eprintln!(
                "  {} {}  {:<12} {}",
                status_icon,
                name_display,
                version_display,
                style("not detected").dim()
            );
        }
    }

    if !found_any {
        ui::warn("No agents detected. Install an agent like Claude Code or Cursor.");
    }

    if !args.all {
        eprintln!();
        eprintln!(
            "Run `{}` to see all {total} supported agents.",
            style("skillx agents --all").cyan()
        );
    }

    Ok(())
}

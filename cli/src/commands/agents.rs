use clap::Args;
use console::style;

use skillx::agent::registry::AgentRegistry;
use skillx::ui;

#[derive(Args, Debug)]
pub struct AgentsArgs {
    /// Show all known agents (including undetected)
    #[arg(long)]
    pub all: bool,
}

pub async fn execute(args: AgentsArgs) -> anyhow::Result<()> {
    let registry = AgentRegistry::new();
    let results = registry.detect_all().await;

    ui::header("Agent Environments");
    eprintln!();

    let mut found_any = false;

    for result in &results {
        if !args.all && !result.detected {
            continue;
        }

        found_any = true;

        let status = if result.detected {
            style("✓ detected").green().to_string()
        } else {
            style("✗ not found").dim().to_string()
        };

        let adapter = registry.get(&result.name);
        let display_name = adapter
            .map(|a| a.display_name())
            .unwrap_or(&result.name);
        let lifecycle = adapter
            .map(|a| format!("{:?}", a.lifecycle_mode()))
            .unwrap_or_default();
        let yolo = adapter.map(|a| a.supports_yolo()).unwrap_or(false);

        eprintln!(
            "  {} [{}]",
            style(display_name).bold(),
            status
        );

        if result.detected {
            if let Some(ref info) = result.info {
                eprintln!("    {}", style(info).dim());
            }
            eprintln!("    Lifecycle: {lifecycle}");
            if yolo {
                let yolo_args = adapter
                    .map(|a| a.yolo_args().join(" "))
                    .unwrap_or_default();
                eprintln!("    YOLO: {yolo_args}");
            }
        }

        eprintln!();
    }

    if !found_any {
        ui::warn("No agents detected. Install an agent like Claude Code or Cursor.");
    }

    Ok(())
}

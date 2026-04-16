use std::time::Duration;

use clap::Args;
use console::style;

use skillx::ui;
use skillx::update_check::{self, InstallMethod, UpdateCheckCache};

#[derive(Args, Debug)]
pub struct UpgradeArgs {}

pub async fn execute(_args: UpgradeArgs) -> anyhow::Result<()> {
    let current = env!("CARGO_PKG_VERSION");
    ui::step(&format!("Current version: v{current}"));
    ui::step("Checking for updates...");

    let latest = update_check::fetch_latest_version(Duration::from_secs(10)).await?;

    // Always update cache when explicitly checking
    update_check::save_cache(&UpdateCheckCache {
        last_checked: chrono::Utc::now(),
        latest_version: latest.clone(),
        current_version: current.to_string(),
    });

    if !update_check::is_newer(current, &latest) {
        ui::success(&format!("You're on the latest version (v{current})."));
        return Ok(());
    }

    let method = update_check::detect_install_method();

    eprintln!(
        "\n  {} {} {}",
        style(format!("v{current}")).red(),
        style("→").dim(),
        style(format!("v{latest}")).green().bold(),
    );
    eprintln!();

    match method {
        InstallMethod::Homebrew => run_upgrade("Homebrew", "brew", &["upgrade", "skillx"]).await,
        InstallMethod::CargoBinstall => {
            run_upgrade("cargo-binstall", "cargo", &["binstall", "skillx", "-y"]).await
        }
        InstallMethod::Cargo => run_upgrade("Cargo", "cargo", &["install", "skillx"]).await,
        InstallMethod::InstallScript => {
            run_upgrade(
                "install.sh",
                "sh",
                &["-c", "curl -fsSL https://skillx.run/install.sh | sh"],
            )
            .await
        }
        InstallMethod::Unknown => {
            ui::warn("Cannot auto-upgrade: unable to determine how skillx was installed.");
            eprintln!();
            eprintln!("  To upgrade, please reinstall using one of the official methods:");
            eprintln!();
            eprintln!("    {}", style("brew install skillx-run/tap/skillx").dim());
            eprintln!("    {}", style("cargo install skillx").dim());
            eprintln!(
                "    {}",
                style("curl -fsSL https://skillx.run/install.sh | sh").dim()
            );
            eprintln!();
            Ok(())
        }
    }
}

async fn run_upgrade(via: &str, cmd: &str, args: &[&str]) -> anyhow::Result<()> {
    ui::step(&format!("Upgrading via {via}..."));

    let status = tokio::process::Command::new(cmd)
        .args(args)
        .status()
        .await?;

    eprintln!();
    if status.success() {
        ui::success(&format!("Successfully upgraded via {via}."));
        Ok(())
    } else {
        anyhow::bail!(
            "upgrade via {via} failed (exit code: {})",
            status.code().unwrap_or(-1)
        );
    }
}

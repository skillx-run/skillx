use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// Print a success message (green checkmark).
pub fn success(msg: &str) {
    eprintln!("{} {}", style("✓").green().bold(), msg);
}

/// Print a warning message (yellow triangle).
pub fn warn(msg: &str) {
    eprintln!("{} {}", style("⚠").yellow().bold(), msg);
}

/// Print an error message (red cross).
pub fn error(msg: &str) {
    eprintln!("{} {}", style("✗").red().bold(), msg);
}

/// Print an info message (blue circle).
pub fn info(msg: &str) {
    eprintln!("{} {}", style("ℹ").blue().bold(), msg);
}

/// Print a step message (cyan arrow) for lifecycle progress.
pub fn step(msg: &str) {
    eprintln!("{} {}", style("→").cyan().bold(), msg);
}

/// Create a spinner with a message.
pub fn spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner:.cyan} {msg}")
            .expect("invalid spinner template"),
    );
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

/// Print a styled header.
pub fn header(msg: &str) {
    eprintln!("\n{}", style(msg).bold().underlined());
}

/// Print a key-value pair.
pub fn kv(key: &str, value: &str) {
    eprintln!("  {}: {}", style(key).dim(), value);
}

use std::io::{BufRead, Write};
use std::path::Path;

use console::style;
use sha2::{Digest, Sha256};

use crate::scanner::{RiskLevel, ScanReport};
use crate::ui;

/// Number of source lines to show before the flagged line.
const CONTEXT_LINES_BEFORE: usize = 3;
/// Number of source lines to show after the flagged line.
const CONTEXT_LINES_AFTER: usize = 2;
/// Width of the separator line in detail view.
const SEPARATOR_WIDTH: usize = 60;

/// Gate scan results: auto-pass PASS/INFO, prompt for WARN, require "yes" for DANGER, block BLOCK.
///
/// `skill_dir` is used to display source context for the `detail` command.
///
/// Note: `auto_yes` only applies to WARN level. DANGER always requires explicit "yes"
/// confirmation to ensure users review dangerous findings before proceeding.
pub fn gate_scan_result(
    scan_report: &Option<ScanReport>,
    skill_dir: &Path,
    auto_yes: bool,
) -> anyhow::Result<()> {
    let report = match scan_report {
        Some(r) => r,
        None => return Ok(()),
    };

    let level = report.overall_level();
    match level {
        RiskLevel::Pass | RiskLevel::Info => {}
        RiskLevel::Warn => {
            if !auto_yes {
                eprint!("{} Continue? [Y/n] ", style("⚠").yellow().bold());
                std::io::stderr().flush().ok();
                let mut input = String::new();
                std::io::stdin().lock().read_line(&mut input)?;
                let input = input.trim().to_lowercase();
                if input == "n" || input == "no" {
                    return Err(crate::error::SkillxError::UserCancelled.into());
                }
            }
        }
        RiskLevel::Danger => {
            eprintln!(
                "\n{}",
                style("DANGER level findings detected. Review carefully.")
                    .red()
                    .bold()
            );
            eprintln!(
                "Type '{}' to see finding details, or type '{}' to continue:",
                style("detail N").cyan(),
                style("yes").green().bold()
            );

            let mut sorted_findings = report.findings.clone();
            sorted_findings.sort_by(|a, b| b.level.cmp(&a.level));

            loop {
                eprint!("{} ", style(">").dim());
                std::io::stderr().flush().ok();
                let mut input = String::new();
                std::io::stdin().lock().read_line(&mut input)?;
                let input = input.trim();

                if input.eq_ignore_ascii_case("yes") {
                    break;
                } else if input.eq_ignore_ascii_case("no") || input.eq_ignore_ascii_case("n") {
                    return Err(crate::error::SkillxError::UserCancelled.into());
                } else if input.starts_with("detail") || input.starts_with("d ") {
                    let num_str = input
                        .strip_prefix("detail")
                        .or_else(|| input.strip_prefix("d "))
                        .unwrap_or("")
                        .trim();
                    if let Ok(n) = num_str.parse::<usize>() {
                        if n > 0 && n <= sorted_findings.len() {
                            let finding = &sorted_findings[n - 1];
                            eprintln!("\n{}", style("─".repeat(SEPARATOR_WIDTH)).dim());
                            eprintln!("  Rule:    {} ({})", finding.rule_id, finding.level);
                            eprintln!("  File:    {}", finding.file);
                            if let Some(line) = finding.line {
                                eprintln!("  Line:    {line}");
                            }
                            eprintln!("  Message: {}", finding.message);

                            if let Some(line) = finding.line {
                                let file_path = skill_dir.join(&finding.file);
                                match std::fs::read_to_string(&file_path) {
                                    Ok(content) => {
                                        let lines: Vec<&str> = content.lines().collect();
                                        let start = line.saturating_sub(CONTEXT_LINES_BEFORE);
                                        let end = (line + CONTEXT_LINES_AFTER).min(lines.len());
                                        eprintln!("\n  Source:");
                                        for (i, l) in lines[start..end].iter().enumerate() {
                                            let line_num = start + i + 1;
                                            let marker = if line_num == line { ">" } else { " " };
                                            eprintln!(
                                                "  {marker} {}: {}",
                                                style(line_num).dim(),
                                                l
                                            );
                                        }
                                    }
                                    Err(_) => {
                                        eprintln!("\n  (source unavailable)");
                                    }
                                }
                            } else {
                                // Binary/resource finding — show file metadata
                                let file_path = skill_dir.join(&finding.file);
                                if file_path.exists() {
                                    if let Ok(meta) = std::fs::metadata(&file_path) {
                                        eprintln!("\n  Size: {}", format_size(meta.len()));
                                    }
                                    if let Ok(content) = std::fs::read(&file_path) {
                                        let hash = format!("{:x}", Sha256::digest(&content));
                                        eprintln!("  SHA-256: {hash}");
                                        if let Some(kind) = infer::get(&content) {
                                            eprintln!(
                                                "  Type: {} ({})",
                                                kind.extension(),
                                                kind.mime_type()
                                            );
                                        }
                                    }
                                }
                            }
                            eprintln!("{}", style("─".repeat(SEPARATOR_WIDTH)).dim());
                        } else {
                            eprintln!(
                                "  Invalid finding number. Valid range: 1-{}",
                                sorted_findings.len()
                            );
                        }
                    } else {
                        eprintln!("  Usage: detail <number>");
                    }
                } else {
                    eprintln!("  Type 'yes' to continue, 'no' to abort, or 'detail N' to inspect");
                }
            }
        }
        RiskLevel::Block => {
            ui::error("BLOCK level findings detected. Execution refused.");
            return Err(crate::error::SkillxError::ScanBlocked.into());
        }
    }
    Ok(())
}

/// Format a byte count into a human-readable size string.
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::{Finding, ScanReport};

    #[test]
    fn test_gate_none_report() {
        let result = gate_scan_result(&None, Path::new("."), false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_gate_pass_auto() {
        let report = ScanReport { findings: vec![] };
        let result = gate_scan_result(&Some(report), Path::new("."), false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_gate_info_auto() {
        let report = ScanReport {
            findings: vec![Finding {
                rule_id: "MD-001".to_string(),
                level: RiskLevel::Info,
                file: "SKILL.md".to_string(),
                line: Some(1),
                message: "info finding".to_string(),
                context: None,
            }],
        };
        let result = gate_scan_result(&Some(report), Path::new("."), false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_gate_block_refused() {
        let report = ScanReport {
            findings: vec![Finding {
                rule_id: "SC-001".to_string(),
                level: RiskLevel::Block,
                file: "run.sh".to_string(),
                line: Some(1),
                message: "blocked".to_string(),
                context: None,
            }],
        };
        let result = gate_scan_result(&Some(report), Path::new("."), false);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.downcast_ref::<crate::error::SkillxError>().is_some());
    }
}

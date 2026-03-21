use std::io::BufRead;
use std::path::Path;

use console::style;

use crate::scanner::{RiskLevel, ScanReport};
use crate::ui;

/// Gate scan results: auto-pass PASS/INFO, prompt for WARN, require "yes" for DANGER, block BLOCK.
///
/// `skill_dir` is used to display source context for the `detail` command.
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
                eprint!(
                    "{} Continue? [Y/n] ",
                    style("⚠").yellow().bold()
                );
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
                style("DANGER level findings detected. Review carefully.").red().bold()
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
                            eprintln!("\n{}", style("─".repeat(60)).dim());
                            eprintln!(
                                "  Rule:    {} ({})",
                                finding.rule_id,
                                finding.level
                            );
                            eprintln!("  File:    {}", finding.file);
                            if let Some(line) = finding.line {
                                eprintln!("  Line:    {line}");
                            }
                            eprintln!("  Message: {}", finding.message);

                            if let Some(line) = finding.line {
                                let file_path = skill_dir.join(&finding.file);
                                if let Ok(content) = std::fs::read_to_string(&file_path) {
                                    let lines: Vec<&str> = content.lines().collect();
                                    let start = line.saturating_sub(3);
                                    let end = (line + 2).min(lines.len());
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
                            }
                            eprintln!("{}", style("─".repeat(60)).dim());
                        } else {
                            eprintln!("  Invalid finding number. Valid range: 1-{}", sorted_findings.len());
                        }
                    } else {
                        eprintln!("  Usage: detail <number>");
                    }
                } else {
                    eprintln!(
                        "  Type 'yes' to continue, 'no' to abort, or 'detail N' to inspect"
                    );
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
        let report = ScanReport {
            findings: vec![],
        };
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

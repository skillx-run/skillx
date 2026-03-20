use regex::Regex;

use super::rules;
use super::{Finding, RiskLevel, ScanReport};

pub struct MarkdownAnalyzer;

impl MarkdownAnalyzer {
    /// Analyze SKILL.md content for security issues.
    pub fn analyze(content: &str, filename: &str) -> ScanReport {
        let mut report = ScanReport::new();

        let checks: Vec<(&str, &[&str], RiskLevel)> = vec![
            ("MD-001", rules::MD_001_PATTERNS, RiskLevel::Danger),
            ("MD-002", rules::MD_002_PATTERNS, RiskLevel::Danger),
            ("MD-003", rules::MD_003_PATTERNS, RiskLevel::Warn),
            ("MD-004", rules::MD_004_PATTERNS, RiskLevel::Warn),
            ("MD-005", rules::MD_005_PATTERNS, RiskLevel::Danger),
            ("MD-006", rules::MD_006_PATTERNS, RiskLevel::Danger),
        ];

        let rule_descriptions: &[(&str, &str)] = &[
            ("MD-001", "prompt injection pattern detected"),
            ("MD-002", "accesses sensitive directories"),
            ("MD-003", "references external data transfer"),
            ("MD-004", "references file/directory deletion"),
            ("MD-005", "references system configuration modification"),
            ("MD-006", "references disabling security checks"),
        ];

        for (rule_id, patterns, level) in &checks {
            for pattern in *patterns {
                if let Ok(re) = Regex::new(pattern) {
                    for (line_num, line) in content.lines().enumerate() {
                        if re.is_match(line) {
                            let desc = rule_descriptions
                                .iter()
                                .find(|(id, _)| id == rule_id)
                                .map(|(_, d)| *d)
                                .unwrap_or("suspicious pattern");

                            report.add(Finding {
                                rule_id: rule_id.to_string(),
                                level: *level,
                                message: format!("{desc}: {}", pattern),
                                file: filename.to_string(),
                                line: Some(line_num + 1),
                                context: Some(line.trim().to_string()),
                            });
                            break; // One finding per pattern per file
                        }
                    }
                }
            }
        }

        report
    }
}

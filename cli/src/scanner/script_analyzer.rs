use std::path::Path;

use crate::error::Result;

use super::binary_analyzer::BinaryAnalyzer;
use super::compiled_rules::SC_RULES;
use super::{Finding, RiskLevel, ScanReport};

pub struct ScriptAnalyzer;

impl ScriptAnalyzer {
    /// Analyze a script file for security issues.
    pub fn analyze(path: &Path, rel_path: &str) -> Result<ScanReport> {
        let mut report = ScanReport::new();

        // SC-001: Binary detection (shared implementation)
        if BinaryAnalyzer::is_executable(path)? {
            report.add(Finding {
                rule_id: "SC-001".to_string(),
                level: RiskLevel::Danger,
                message: "binary executable detected in scripts".to_string(),
                file: rel_path.to_string(),
                line: None,
                context: None,
            });
            return Ok(report);
        }

        // Read content for text analysis
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => {
                // If we can't read as text, it might be binary
                report.add(Finding {
                    rule_id: "SC-001".to_string(),
                    level: RiskLevel::Danger,
                    message: "file cannot be read as text (possibly binary)".to_string(),
                    file: rel_path.to_string(),
                    line: None,
                    context: None,
                });
                return Ok(report);
            }
        };

        // SC-002 through SC-011: Pre-compiled pattern matching
        for rule in SC_RULES.iter() {
            for re in &rule.patterns {
                for (line_num, line) in content.lines().enumerate() {
                    if re.is_match(line) {
                        report.add(Finding {
                            rule_id: rule.id.to_string(),
                            level: rule.level,
                            message: format!("{}: {}", rule.description, re.as_str()),
                            file: rel_path.to_string(),
                            line: Some(line_num + 1),
                            context: Some(line.trim().to_string()),
                        });
                        break;
                    }
                }
            }
        }

        Ok(report)
    }
}

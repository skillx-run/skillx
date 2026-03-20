use super::compiled_rules::MD_RULES;
use super::{Finding, ScanReport};

pub struct MarkdownAnalyzer;

impl MarkdownAnalyzer {
    /// Analyze SKILL.md content for security issues.
    pub fn analyze(content: &str, filename: &str) -> ScanReport {
        let mut report = ScanReport::new();

        for rule in MD_RULES.iter() {
            for re in &rule.patterns {
                for (line_num, line) in content.lines().enumerate() {
                    if re.is_match(line) {
                        report.add(Finding {
                            rule_id: rule.id.to_string(),
                            level: rule.level,
                            message: format!("{}: {}", rule.description, re.as_str()),
                            file: filename.to_string(),
                            line: Some(line_num + 1),
                            context: Some(line.trim().to_string()),
                        });
                        break; // One finding per pattern per file
                    }
                }
            }
        }

        report
    }
}

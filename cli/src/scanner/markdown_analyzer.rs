use super::compiled_rules::MD_RULES;
use super::{Finding, RiskLevel, ScanReport};

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

        // MD-007: License not declared in frontmatter (INFO)
        if Self::has_frontmatter_without_license(content) {
            report.add(Finding {
                rule_id: "MD-007".to_string(),
                level: RiskLevel::Info,
                message: "license not declared in frontmatter".to_string(),
                file: filename.to_string(),
                line: Some(1),
                context: None,
            });
        }

        report
    }

    /// Check if the content has a YAML frontmatter block but no `license` field.
    fn has_frontmatter_without_license(content: &str) -> bool {
        let trimmed = content.trim_start();
        let after_first = match trimmed.strip_prefix("---") {
            Some(rest) => rest,
            None => return false, // No frontmatter at all — don't trigger
        };

        let end = match after_first.find("\n---") {
            Some(pos) => pos,
            None => return false, // Unclosed frontmatter — don't trigger
        };

        let yaml = &after_first[..end];
        // Check if any line starts with "license:" (case-insensitive)
        !yaml
            .lines()
            .any(|line| line.trim_start().to_lowercase().starts_with("license:"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_md007_triggers_when_frontmatter_has_no_license() {
        let content = "---\nname: my-skill\nversion: 1.0\n---\n# Skill";
        let report = MarkdownAnalyzer::analyze(content, "SKILL.md");
        let md007: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.rule_id == "MD-007")
            .collect();
        assert_eq!(md007.len(), 1);
        assert_eq!(md007[0].level, RiskLevel::Info);
    }

    #[test]
    fn test_md007_does_not_trigger_when_license_present() {
        let content = "---\nname: my-skill\nlicense: MIT\n---\n# Skill";
        let report = MarkdownAnalyzer::analyze(content, "SKILL.md");
        let md007: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.rule_id == "MD-007")
            .collect();
        assert!(md007.is_empty());
    }

    #[test]
    fn test_md007_does_not_trigger_without_frontmatter() {
        let content = "# My Skill\nSome content here.";
        let report = MarkdownAnalyzer::analyze(content, "SKILL.md");
        let md007: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.rule_id == "MD-007")
            .collect();
        assert!(md007.is_empty());
    }

    #[test]
    fn test_md007_case_insensitive_license() {
        let content = "---\nname: test\nLicense: Apache-2.0\n---\n# Skill";
        let report = MarkdownAnalyzer::analyze(content, "SKILL.md");
        let md007: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.rule_id == "MD-007")
            .collect();
        assert!(md007.is_empty());
    }
}

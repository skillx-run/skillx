use console::style;

use super::{Finding, RiskLevel, ScanReport};

/// Format a scan report as colored terminal text.
pub struct TextFormatter;

impl TextFormatter {
    pub fn format(report: &ScanReport) -> String {
        let mut output = String::new();

        let overall = report.overall_level();
        output.push_str(&format!("\nScan Result: {}\n", Self::styled_level(overall)));

        if report.findings.is_empty() {
            output.push_str(&format!(
                "  {} No security issues found.\n",
                style("✓").green().bold()
            ));
            return output;
        }

        output.push_str(&format!(
            "  Found {} issue(s):\n\n",
            report.findings.len()
        ));

        // Group by level (highest first)
        let mut sorted = report.findings.clone();
        sorted.sort_by(|a, b| b.level.cmp(&a.level));

        for (i, finding) in sorted.iter().enumerate() {
            output.push_str(&Self::format_finding(i + 1, finding));
            output.push('\n');
        }

        output
    }

    fn format_finding(index: usize, finding: &Finding) -> String {
        let mut s = String::new();

        s.push_str(&format!(
            "  {}. [{}] {} ({})\n",
            index,
            Self::styled_level(finding.level),
            finding.message,
            finding.rule_id,
        ));

        s.push_str(&format!(
            "     File: {}",
            finding.file,
        ));
        if let Some(line) = finding.line {
            s.push_str(&format!(":{line}"));
        }
        s.push('\n');

        if let Some(ref ctx) = finding.context {
            let truncated = if ctx.len() > 120 {
                format!("{}...", &ctx[..117])
            } else {
                ctx.clone()
            };
            s.push_str(&format!(
                "     Context: {}\n",
                style(&truncated).dim()
            ));
        }

        s
    }

    fn styled_level(level: RiskLevel) -> String {
        match level {
            RiskLevel::Pass => style("PASS").green().bold().to_string(),
            RiskLevel::Info => style("INFO").blue().bold().to_string(),
            RiskLevel::Warn => style("WARN").yellow().bold().to_string(),
            RiskLevel::Danger => style("DANGER").red().bold().to_string(),
            RiskLevel::Block => style("BLOCK").red().bold().reverse().to_string(),
        }
    }
}

/// Format a scan report as JSON.
pub struct JsonFormatter;

impl JsonFormatter {
    pub fn format(report: &ScanReport) -> String {
        serde_json::to_string_pretty(report).unwrap_or_else(|_| "{}".to_string())
    }
}

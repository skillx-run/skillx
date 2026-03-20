use regex::Regex;
use std::path::Path;

use crate::error::{Result, SkillxError};

use super::rules;
use super::{Finding, RiskLevel, ScanReport};

pub struct ScriptAnalyzer;

impl ScriptAnalyzer {
    /// Analyze a script file for security issues.
    pub fn analyze(path: &Path, rel_path: &str) -> Result<ScanReport> {
        let mut report = ScanReport::new();

        // SC-001: Binary detection
        if Self::is_binary(path)? {
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

        // SC-002 through SC-011: Pattern matching
        let checks: Vec<(&str, &[&str], RiskLevel, &str)> = vec![
            (
                "SC-002",
                rules::SC_002_PATTERNS,
                RiskLevel::Danger,
                "dynamic code execution",
            ),
            (
                "SC-003",
                rules::SC_003_PATTERNS,
                RiskLevel::Danger,
                "recursive file deletion",
            ),
            (
                "SC-004",
                rules::SC_004_PATTERNS,
                RiskLevel::Danger,
                "accesses sensitive directories",
            ),
            (
                "SC-005",
                rules::SC_005_PATTERNS,
                RiskLevel::Danger,
                "modifies shell configuration",
            ),
            (
                "SC-006",
                rules::SC_006_PATTERNS,
                RiskLevel::Warn,
                "network request detected",
            ),
            (
                "SC-007",
                rules::SC_007_PATTERNS,
                RiskLevel::Warn,
                "writes outside skill directory",
            ),
            (
                "SC-008",
                rules::SC_008_PATTERNS,
                RiskLevel::Warn,
                "privilege escalation attempt",
            ),
            (
                "SC-009",
                rules::SC_009_PATTERNS,
                RiskLevel::Danger,
                "setuid/setgid permission change",
            ),
            (
                "SC-010",
                rules::SC_010_PATTERNS,
                RiskLevel::Block,
                "self-replication detected",
            ),
            (
                "SC-011",
                rules::SC_011_PATTERNS,
                RiskLevel::Block,
                "modifies skillx paths",
            ),
        ];

        for (rule_id, patterns, level, description) in &checks {
            for pattern in *patterns {
                if let Ok(re) = Regex::new(pattern) {
                    for (line_num, line) in content.lines().enumerate() {
                        if re.is_match(line) {
                            report.add(Finding {
                                rule_id: rule_id.to_string(),
                                level: *level,
                                message: format!("{description}: {pattern}"),
                                file: rel_path.to_string(),
                                line: Some(line_num + 1),
                                context: Some(line.trim().to_string()),
                            });
                            break;
                        }
                    }
                }
            }
        }

        Ok(report)
    }

    /// Check if a file is a binary executable using magic bytes.
    fn is_binary(path: &Path) -> Result<bool> {
        let buf = std::fs::read(path)
            .map_err(|e| SkillxError::Scan(format!("failed to read file: {e}")))?;

        if buf.len() < 4 {
            return Ok(false);
        }

        // Check via infer crate
        if let Some(kind) = infer::get(&buf) {
            let mime = kind.mime_type();
            if mime.starts_with("application/x-executable")
                || mime == "application/x-mach-binary"
                || mime == "application/x-elf"
                || mime == "application/vnd.microsoft.portable-executable"
                || mime == "application/x-sharedlib"
            {
                return Ok(true);
            }
        }

        // Direct magic byte checks
        // ELF
        if buf.starts_with(b"\x7fELF") {
            return Ok(true);
        }
        // Mach-O
        if buf.starts_with(&[0xfe, 0xed, 0xfa, 0xce])
            || buf.starts_with(&[0xfe, 0xed, 0xfa, 0xcf])
            || buf.starts_with(&[0xce, 0xfa, 0xed, 0xfe])
            || buf.starts_with(&[0xcf, 0xfa, 0xed, 0xfe])
        {
            return Ok(true);
        }
        // PE (Windows)
        if buf.starts_with(b"MZ") {
            return Ok(true);
        }

        Ok(false)
    }
}

use std::path::Path;

use crate::error::{Result, SkillxError};

use super::compiled_rules::{SC_RULES, SC_RULE_LEVELS};
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

        // SC-002 through SC-011: Pre-compiled pattern matching
        for rule in SC_RULES.iter() {
            let level = SC_RULE_LEVELS
                .iter()
                .find(|(id, _)| *id == rule.id)
                .map(|(_, l)| *l)
                .unwrap_or(RiskLevel::Warn);

            for re in &rule.patterns {
                for (line_num, line) in content.lines().enumerate() {
                    if re.is_match(line) {
                        report.add(Finding {
                            rule_id: rule.id.to_string(),
                            level,
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

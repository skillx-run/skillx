use std::path::Path;

use crate::error::{Result, SkillxError};

use super::rules;
use super::{Finding, RiskLevel, ScanReport};

pub struct ResourceAnalyzer;

impl ResourceAnalyzer {
    /// Analyze a resource file for security issues.
    pub fn analyze(path: &Path, rel_path: &str) -> Result<ScanReport> {
        let mut report = ScanReport::new();

        let metadata = std::fs::metadata(path)
            .map_err(|e| SkillxError::Scan(format!("failed to read metadata: {e}")))?;

        // RS-002: Large file check
        let size = metadata.len();
        if size > rules::RS_002_SIZE_THRESHOLD {
            report.add(Finding {
                rule_id: "RS-002".to_string(),
                level: RiskLevel::Info,
                message: format!("large file: {} MB", size / (1024 * 1024)),
                file: rel_path.to_string(),
                line: None,
                context: None,
            });
        }

        // RS-001: Extension vs magic bytes mismatch (disguised file)
        if let Some(mismatch) = Self::check_extension_mismatch(path)? {
            report.add(Finding {
                rule_id: "RS-001".to_string(),
                level: RiskLevel::Warn,
                message: format!("extension mismatch: {mismatch}"),
                file: rel_path.to_string(),
                line: None,
                context: None,
            });
        }

        // RS-003: Executable in references/
        if rel_path.starts_with("references/") || rel_path.starts_with("references\\") {
            if Self::is_executable(path)? {
                report.add(Finding {
                    rule_id: "RS-003".to_string(),
                    level: RiskLevel::Danger,
                    message: "executable file in references directory".to_string(),
                    file: rel_path.to_string(),
                    line: None,
                    context: None,
                });
            }
        }

        Ok(report)
    }

    /// Check if file extension matches its actual content type.
    fn check_extension_mismatch(path: &Path) -> Result<Option<String>> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        if ext.is_empty() {
            return Ok(None);
        }

        let buf = std::fs::read(path)
            .map_err(|e| SkillxError::Scan(format!("failed to read file: {e}")))?;

        if buf.len() < 4 {
            return Ok(None);
        }

        let actual_kind = infer::get(&buf);

        // Check common mismatches
        let claimed_types: &[(&str, &[&str])] = &[
            ("pdf", &["application/pdf"]),
            ("png", &["image/png"]),
            ("jpg", &["image/jpeg"]),
            ("jpeg", &["image/jpeg"]),
            ("gif", &["image/gif"]),
            ("zip", &["application/zip"]),
            ("gz", &["application/gzip"]),
            ("tar", &["application/x-tar"]),
            ("doc", &["application/msword", "application/vnd.openxmlformats"]),
            ("docx", &["application/vnd.openxmlformats", "application/zip"]),
            ("txt", &[]), // text has no magic bytes
        ];

        if let Some((_, expected_mimes)) = claimed_types.iter().find(|(e, _)| *e == ext) {
            if expected_mimes.is_empty() {
                return Ok(None); // Can't validate text files
            }

            if let Some(kind) = actual_kind {
                let actual_mime = kind.mime_type();
                if !expected_mimes.iter().any(|m| actual_mime.starts_with(m)) {
                    return Ok(Some(format!(
                        "claims to be .{ext} but detected as {actual_mime}"
                    )));
                }
            }
        }

        Ok(None)
    }

    /// Check if a file is executable (binary).
    fn is_executable(path: &Path) -> Result<bool> {
        let buf = std::fs::read(path)
            .map_err(|e| SkillxError::Scan(format!("failed to read file: {e}")))?;

        if buf.len() < 4 {
            return Ok(false);
        }

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
        // PE
        if buf.starts_with(b"MZ") {
            return Ok(true);
        }

        Ok(false)
    }
}

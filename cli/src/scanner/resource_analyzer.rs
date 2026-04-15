use std::path::Path;

use crate::error::{Result, SkillxError};

use super::binary_analyzer::BinaryAnalyzer;
use super::rules;
use super::{Finding, RiskLevel, ScanReport};

pub struct ResourceAnalyzer;

impl ResourceAnalyzer {
    /// Analyze a resource file for security issues.
    pub fn analyze(path: &Path, rel_path: &str) -> Result<ScanReport> {
        let mut report = ScanReport::new();

        // RS-004: Symlink detection (defense in depth — also checked in scan_directory)
        if path.is_symlink() {
            let target = std::fs::read_link(path)
                .map(|t| t.to_string_lossy().to_string())
                .unwrap_or_else(|_| "unknown".to_string());
            report.add(Finding {
                rule_id: "RS-004".to_string(),
                level: RiskLevel::Danger,
                message: format!("symlink detected pointing to: {target}"),
                file: rel_path.to_string(),
                line: None,
                context: None,
            });
            return Ok(report); // Don't follow or analyze symlink target
        }

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
        // Only read the first few KB for detection, not the whole file.
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

        // RS-003: Executable in references/ (shared detection)
        let in_references =
            rel_path.starts_with("references/") || rel_path.starts_with("references\\");
        if in_references && BinaryAnalyzer::is_executable(path)? {
            report.add(Finding {
                rule_id: "RS-003".to_string(),
                level: RiskLevel::Danger,
                message: "executable file in references directory".to_string(),
                file: rel_path.to_string(),
                line: None,
                context: None,
            });
        }

        // RS-005: Script file in references/ (shebang detection)
        if in_references && !BinaryAnalyzer::is_executable(path)? {
            if let Ok(bytes) = std::fs::read(path) {
                if bytes.starts_with(b"#!") {
                    report.add(Finding {
                        rule_id: "RS-005".to_string(),
                        level: RiskLevel::Warn,
                        message: "script file (shebang detected) in references directory"
                            .to_string(),
                        file: rel_path.to_string(),
                        line: None,
                        context: None,
                    });
                }
            }
        }

        Ok(report)
    }

    /// Check if file extension matches its actual content type.
    /// Only reads first few KB to avoid loading large files.
    fn check_extension_mismatch(path: &Path) -> Result<Option<String>> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        if ext.is_empty() {
            return Ok(None);
        }

        let buf = BinaryAnalyzer::read_magic_bytes(path)?;

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
            (
                "doc",
                &["application/msword", "application/vnd.openxmlformats"],
            ),
            (
                "docx",
                &["application/vnd.openxmlformats", "application/zip"],
            ),
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
}

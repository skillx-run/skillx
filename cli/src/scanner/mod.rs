pub mod binary_analyzer;
pub mod compiled_rules;
pub mod markdown_analyzer;
pub mod normalize;
pub mod report;
pub mod resource_analyzer;
pub mod rules;
pub mod script_analyzer;

use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;

use crate::error::Result;

/// Risk level for scan findings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Pass,
    Info,
    Warn,
    Danger,
    Block,
}

impl fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RiskLevel::Pass => write!(f, "PASS"),
            RiskLevel::Info => write!(f, "INFO"),
            RiskLevel::Warn => write!(f, "WARN"),
            RiskLevel::Danger => write!(f, "DANGER"),
            RiskLevel::Block => write!(f, "BLOCK"),
        }
    }
}

impl std::str::FromStr for RiskLevel {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pass" => Ok(RiskLevel::Pass),
            "info" => Ok(RiskLevel::Info),
            "warn" => Ok(RiskLevel::Warn),
            "danger" => Ok(RiskLevel::Danger),
            "block" => Ok(RiskLevel::Block),
            _ => Err(format!("invalid risk level: '{s}'")),
        }
    }
}

/// A single finding from the scan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub rule_id: String,
    pub level: RiskLevel,
    pub message: String,
    pub file: String,
    pub line: Option<usize>,
    pub context: Option<String>,
}

/// Scan report containing all findings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScanReport {
    pub findings: Vec<Finding>,
}

impl ScanReport {
    pub fn new() -> Self {
        Self::default()
    }

    /// Overall risk level (max of all findings).
    pub fn overall_level(&self) -> RiskLevel {
        self.findings
            .iter()
            .map(|f| f.level)
            .max()
            .unwrap_or(RiskLevel::Pass)
    }

    pub fn add(&mut self, finding: Finding) {
        self.findings.push(finding);
    }

    pub fn merge(&mut self, other: ScanReport) {
        self.findings.extend(other.findings);
    }
}

/// The main scan engine that orchestrates all analyzers.
pub struct ScanEngine;

impl ScanEngine {
    /// Scan a skill directory and return a report.
    pub fn scan(skill_dir: &Path) -> Result<ScanReport> {
        let mut report = ScanReport::new();

        // Scan SKILL.md
        let skill_md = skill_dir.join("SKILL.md");
        if skill_md.exists() {
            let content = std::fs::read_to_string(&skill_md).map_err(|e| {
                crate::error::SkillxError::Scan(format!("failed to read SKILL.md: {e}"))
            })?;
            let md_report = markdown_analyzer::MarkdownAnalyzer::analyze(&content, "SKILL.md");
            report.merge(md_report);
        }

        // Scan scripts
        let scripts_dir = skill_dir.join("scripts");
        if scripts_dir.is_dir() {
            Self::scan_directory(&scripts_dir, skill_dir, &mut report, true)?;
        }

        // Scan references
        let refs_dir = skill_dir.join("references");
        if refs_dir.is_dir() {
            Self::scan_directory(&refs_dir, skill_dir, &mut report, false)?;
        }

        // Also scan any other files in root (not SKILL.md which is already scanned)
        Self::scan_root_files(skill_dir, &mut report)?;

        Ok(report)
    }

    fn scan_directory(
        dir: &Path,
        skill_dir: &Path,
        report: &mut ScanReport,
        is_scripts: bool,
    ) -> Result<()> {
        let entries = std::fs::read_dir(dir)
            .map_err(|e| crate::error::SkillxError::Scan(format!("failed to read dir: {e}")))?;

        for entry in entries {
            let entry = entry
                .map_err(|e| crate::error::SkillxError::Scan(format!("dir entry error: {e}")))?;
            let path = entry.path();
            let file_type = entry.file_type().map_err(|e| {
                crate::error::SkillxError::Scan(format!("failed to get file type: {e}"))
            })?;
            let rel_path = path
                .strip_prefix(skill_dir)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();

            // RS-004: Symlink detection — check BEFORE is_dir/is_file to prevent traversal
            if file_type.is_symlink() {
                let target = std::fs::read_link(&path)
                    .map(|t| t.to_string_lossy().to_string())
                    .unwrap_or_else(|_| "unknown".to_string());
                report.add(Finding {
                    rule_id: "RS-004".to_string(),
                    level: RiskLevel::Danger,
                    message: format!("symlink detected pointing to: {target}"),
                    file: rel_path,
                    line: None,
                    context: None,
                });
                continue; // Do NOT follow symlinks
            }

            if file_type.is_dir() {
                Self::scan_directory(&path, skill_dir, report, is_scripts)?;
                continue;
            }

            if is_scripts {
                // Script analysis (binary detection + content analysis)
                let script_report = script_analyzer::ScriptAnalyzer::analyze(&path, &rel_path)?;
                report.merge(script_report);
            } else {
                // Resource analysis
                let res_report = resource_analyzer::ResourceAnalyzer::analyze(&path, &rel_path)?;
                report.merge(res_report);
            }
        }

        Ok(())
    }

    fn scan_root_files(skill_dir: &Path, report: &mut ScanReport) -> Result<()> {
        let entries = std::fs::read_dir(skill_dir)
            .map_err(|e| crate::error::SkillxError::Scan(format!("failed to read dir: {e}")))?;

        for entry in entries {
            let entry = entry
                .map_err(|e| crate::error::SkillxError::Scan(format!("dir entry error: {e}")))?;
            let path = entry.path();
            let file_type = entry.file_type().map_err(|e| {
                crate::error::SkillxError::Scan(format!("failed to get file type: {e}"))
            })?;

            // RS-004: Symlink detection
            if file_type.is_symlink() {
                let rel_path = path
                    .strip_prefix(skill_dir)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .to_string();
                let target = std::fs::read_link(&path)
                    .map(|t| t.to_string_lossy().to_string())
                    .unwrap_or_else(|_| "unknown".to_string());
                report.add(Finding {
                    rule_id: "RS-004".to_string(),
                    level: RiskLevel::Danger,
                    message: format!("symlink detected pointing to: {target}"),
                    file: rel_path,
                    line: None,
                    context: None,
                });
                continue;
            }

            if file_type.is_dir() {
                continue;
            }
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name == "SKILL.md" {
                continue; // Already scanned
            }

            let rel_path = path
                .strip_prefix(skill_dir)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();

            // Check if it's a script-like file (by extension or shebang)
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            let is_script_ext = matches!(
                ext,
                "py" | "sh" | "bash" | "js" | "ts" | "rb" | "pl" | "ps1"
            );

            // Shebang detection for extensionless files
            // Only read first few bytes — no need to load entire file
            let has_shebang = if !is_script_ext && ext.is_empty() {
                binary_analyzer::BinaryAnalyzer::read_magic_bytes(&path)
                    .ok()
                    .map(|bytes| bytes.starts_with(b"#!"))
                    .unwrap_or(false)
            } else {
                false
            };

            if is_script_ext || has_shebang {
                let script_report = script_analyzer::ScriptAnalyzer::analyze(&path, &rel_path)?;
                report.merge(script_report);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn test_symlink_to_file_detected() {
        let dir = tempfile::tempdir().unwrap();
        let skill_dir = dir.path();

        // Create minimal SKILL.md
        std::fs::write(skill_dir.join("SKILL.md"), "---\nname: test\n---\n# Test\n").unwrap();

        // Create scripts/ with a symlink
        let scripts = skill_dir.join("scripts");
        std::fs::create_dir_all(&scripts).unwrap();
        std::os::unix::fs::symlink("/etc/passwd", scripts.join("secret")).unwrap();

        let report = ScanEngine::scan(skill_dir).unwrap();
        let rs004: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.rule_id == "RS-004")
            .collect();
        assert!(
            !rs004.is_empty(),
            "RS-004 should detect symlinks in scripts/"
        );
        assert_eq!(rs004[0].level, RiskLevel::Danger);
        assert!(rs004[0].message.contains("/etc/passwd"));
    }

    #[cfg(unix)]
    #[test]
    fn test_symlink_dir_not_traversed() {
        let dir = tempfile::tempdir().unwrap();
        let skill_dir = dir.path();

        std::fs::write(skill_dir.join("SKILL.md"), "---\nname: test\n---\n# Test\n").unwrap();

        // Create a symlink directory pointing to /etc
        let scripts = skill_dir.join("scripts");
        std::fs::create_dir_all(&scripts).unwrap();
        std::os::unix::fs::symlink("/etc", scripts.join("etc_link")).unwrap();

        let report = ScanEngine::scan(skill_dir).unwrap();
        let rs004: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.rule_id == "RS-004")
            .collect();
        assert!(
            !rs004.is_empty(),
            "RS-004 should detect symlink directories"
        );

        // Verify no findings from files inside /etc (symlink was NOT followed)
        let etc_findings: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.file.contains("etc_link/"))
            .collect();
        assert!(
            etc_findings.is_empty(),
            "Scanner should not traverse into symlinked directories"
        );
    }

    #[test]
    fn test_regular_file_no_rs004() {
        let dir = tempfile::tempdir().unwrap();
        let skill_dir = dir.path();

        std::fs::write(skill_dir.join("SKILL.md"), "---\nname: test\n---\n# Test\n").unwrap();
        let scripts = skill_dir.join("scripts");
        std::fs::create_dir_all(&scripts).unwrap();
        std::fs::write(scripts.join("safe.sh"), "#!/bin/bash\necho hello\n").unwrap();

        let report = ScanEngine::scan(skill_dir).unwrap();
        let rs004: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.rule_id == "RS-004")
            .collect();
        assert!(rs004.is_empty(), "RS-004 should not fire on regular files");
    }

    // ── Shebang detection ──

    #[test]
    fn test_extensionless_shebang_scanned() {
        let dir = tempfile::tempdir().unwrap();
        let skill_dir = dir.path();

        std::fs::write(skill_dir.join("SKILL.md"), "---\nname: test\n---\n# Test\n").unwrap();
        // Extensionless file with shebang and dangerous content
        std::fs::write(skill_dir.join("runner"), "#!/bin/bash\neval(\"exploit\")\n").unwrap();

        let report = ScanEngine::scan(skill_dir).unwrap();
        let sc002: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.rule_id == "SC-002")
            .collect();
        assert!(
            !sc002.is_empty(),
            "Extensionless files with shebang should be scanned as scripts"
        );
    }

    #[test]
    fn test_extensionless_no_shebang_skipped() {
        let dir = tempfile::tempdir().unwrap();
        let skill_dir = dir.path();

        std::fs::write(skill_dir.join("SKILL.md"), "---\nname: test\n---\n# Test\n").unwrap();
        // Extensionless file WITHOUT shebang — should not be scanned as script
        std::fs::write(skill_dir.join("data"), "eval(\"exploit\")\n").unwrap();

        let report = ScanEngine::scan(skill_dir).unwrap();
        let sc002: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.rule_id == "SC-002")
            .collect();
        assert!(
            sc002.is_empty(),
            "Extensionless files without shebang should not be scanned as scripts"
        );
    }

    // ── RS-005: Script in references/ ──

    #[test]
    fn test_references_script_detected() {
        let dir = tempfile::tempdir().unwrap();
        let skill_dir = dir.path();

        std::fs::write(skill_dir.join("SKILL.md"), "---\nname: test\n---\n# Test\n").unwrap();
        let refs = skill_dir.join("references");
        std::fs::create_dir_all(&refs).unwrap();
        std::fs::write(refs.join("helper.sh"), "#!/bin/bash\necho hello\n").unwrap();

        let report = ScanEngine::scan(skill_dir).unwrap();
        let rs005: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.rule_id == "RS-005")
            .collect();
        assert!(
            !rs005.is_empty(),
            "RS-005 should detect script files in references/"
        );
        assert_eq!(rs005[0].level, RiskLevel::Warn);
    }

    #[test]
    fn test_references_text_no_rs005() {
        let dir = tempfile::tempdir().unwrap();
        let skill_dir = dir.path();

        std::fs::write(skill_dir.join("SKILL.md"), "---\nname: test\n---\n# Test\n").unwrap();
        let refs = skill_dir.join("references");
        std::fs::create_dir_all(&refs).unwrap();
        std::fs::write(refs.join("notes.txt"), "Just some notes.\n").unwrap();

        let report = ScanEngine::scan(skill_dir).unwrap();
        let rs005: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.rule_id == "RS-005")
            .collect();
        assert!(
            rs005.is_empty(),
            "RS-005 should not fire on plain text files"
        );
    }
}

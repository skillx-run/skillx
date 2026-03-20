pub mod binary_analyzer;
pub mod markdown_analyzer;
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanReport {
    pub findings: Vec<Finding>,
}

impl ScanReport {
    pub fn new() -> Self {
        ScanReport {
            findings: Vec::new(),
        }
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
            let content = std::fs::read_to_string(&skill_md)
                .map_err(|e| crate::error::SkillxError::Scan(format!("failed to read SKILL.md: {e}")))?;
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
            let rel_path = path
                .strip_prefix(skill_dir)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();

            if path.is_dir() {
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
            if path.is_dir() {
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

            // Check if it's a script-like file
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if matches!(ext, "py" | "sh" | "bash" | "js" | "ts" | "rb" | "pl" | "ps1") {
                let script_report = script_analyzer::ScriptAnalyzer::analyze(&path, &rel_path)?;
                report.merge(script_report);
            }
        }

        Ok(())
    }
}

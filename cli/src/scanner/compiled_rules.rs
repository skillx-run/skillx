use regex::Regex;
use std::sync::LazyLock;

use super::rules;
use super::RiskLevel;

/// A pre-compiled rule with level embedded.
pub struct CompiledRule {
    pub id: &'static str,
    pub patterns: Vec<Regex>,
    pub level: RiskLevel,
    pub description: &'static str,
}

fn compile_patterns(patterns: &[&str]) -> Vec<Regex> {
    patterns
        .iter()
        .map(|p| {
            Regex::new(p)
                .unwrap_or_else(|e| panic!("BUG: failed to compile scanner regex '{p}': {e}"))
        })
        .collect()
}

// --- Markdown rules ---

pub static MD_RULES: LazyLock<Vec<CompiledRule>> = LazyLock::new(|| {
    vec![
        CompiledRule {
            id: "MD-001",
            patterns: compile_patterns(rules::MD_001_PATTERNS),
            level: RiskLevel::Danger,
            description: "prompt injection pattern detected",
        },
        CompiledRule {
            id: "MD-002",
            patterns: compile_patterns(rules::MD_002_PATTERNS),
            level: RiskLevel::Danger,
            description: "accesses sensitive directories",
        },
        CompiledRule {
            id: "MD-003",
            patterns: compile_patterns(rules::MD_003_PATTERNS),
            level: RiskLevel::Warn,
            description: "references external data transfer",
        },
        CompiledRule {
            id: "MD-004",
            patterns: compile_patterns(rules::MD_004_PATTERNS),
            level: RiskLevel::Warn,
            description: "references file/directory deletion",
        },
        CompiledRule {
            id: "MD-005",
            patterns: compile_patterns(rules::MD_005_PATTERNS),
            level: RiskLevel::Danger,
            description: "references system configuration modification",
        },
        CompiledRule {
            id: "MD-006",
            patterns: compile_patterns(rules::MD_006_PATTERNS),
            level: RiskLevel::Danger,
            description: "references disabling security checks",
        },
        CompiledRule {
            id: "MD-010",
            patterns: compile_patterns(rules::MD_010_PATTERNS),
            level: RiskLevel::Warn,
            description: "hidden text or invisible characters",
        },
        CompiledRule {
            id: "MD-011",
            patterns: compile_patterns(rules::MD_011_PATTERNS),
            level: RiskLevel::Warn,
            description: "data URI or JavaScript URI scheme",
        },
    ]
});

// --- Script rules ---

pub static SC_RULES: LazyLock<Vec<CompiledRule>> = LazyLock::new(|| {
    vec![
        CompiledRule {
            id: "SC-002",
            patterns: compile_patterns(rules::SC_002_PATTERNS),
            level: RiskLevel::Danger,
            description: "dynamic code execution",
        },
        CompiledRule {
            id: "SC-003",
            patterns: compile_patterns(rules::SC_003_PATTERNS),
            level: RiskLevel::Danger,
            description: "recursive file deletion",
        },
        CompiledRule {
            id: "SC-004",
            patterns: compile_patterns(rules::SC_004_PATTERNS),
            level: RiskLevel::Danger,
            description: "accesses sensitive directories",
        },
        CompiledRule {
            id: "SC-005",
            patterns: compile_patterns(rules::SC_005_PATTERNS),
            level: RiskLevel::Danger,
            description: "modifies shell configuration",
        },
        CompiledRule {
            id: "SC-006",
            patterns: compile_patterns(rules::SC_006_PATTERNS),
            level: RiskLevel::Warn,
            description: "network request detected",
        },
        CompiledRule {
            id: "SC-007",
            patterns: compile_patterns(rules::SC_007_PATTERNS),
            level: RiskLevel::Warn,
            description: "writes outside skill directory",
        },
        CompiledRule {
            id: "SC-008",
            patterns: compile_patterns(rules::SC_008_PATTERNS),
            level: RiskLevel::Warn,
            description: "privilege escalation attempt",
        },
        CompiledRule {
            id: "SC-009",
            patterns: compile_patterns(rules::SC_009_PATTERNS),
            level: RiskLevel::Danger,
            description: "setuid/setgid permission change",
        },
        CompiledRule {
            id: "SC-010",
            patterns: compile_patterns(rules::SC_010_PATTERNS),
            level: RiskLevel::Block,
            description: "self-replication detected",
        },
        CompiledRule {
            id: "SC-011",
            patterns: compile_patterns(rules::SC_011_PATTERNS),
            level: RiskLevel::Block,
            description: "modifies skillx paths",
        },
        CompiledRule {
            id: "SC-012",
            patterns: compile_patterns(rules::SC_012_PATTERNS),
            level: RiskLevel::Danger,
            description: "base64 decode execution",
        },
        CompiledRule {
            id: "SC-013",
            patterns: compile_patterns(rules::SC_013_PATTERNS),
            level: RiskLevel::Danger,
            description: "hex-encoded execution",
        },
        CompiledRule {
            id: "SC-014",
            patterns: compile_patterns(rules::SC_014_PATTERNS),
            level: RiskLevel::Warn,
            description: "string concatenation obfuscation",
        },
        CompiledRule {
            id: "SC-015",
            patterns: compile_patterns(rules::SC_015_PATTERNS),
            level: RiskLevel::Danger,
            description: "environment variable exfiltration",
        },
    ]
});

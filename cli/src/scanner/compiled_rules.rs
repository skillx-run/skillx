use regex::Regex;
use std::sync::LazyLock;

use super::rules;

/// A pre-compiled rule: (rule_id, compiled regexes, risk_level, description).
pub struct CompiledRule {
    pub id: &'static str,
    pub patterns: Vec<Regex>,
    pub description: &'static str,
}

fn compile_patterns(patterns: &[&str]) -> Vec<Regex> {
    patterns
        .iter()
        .filter_map(|p| Regex::new(p).ok())
        .collect()
}

// --- Markdown rules ---

pub static MD_RULES: LazyLock<Vec<CompiledRule>> = LazyLock::new(|| {
    vec![
        CompiledRule {
            id: "MD-001",
            patterns: compile_patterns(rules::MD_001_PATTERNS),
            description: "prompt injection pattern detected",
        },
        CompiledRule {
            id: "MD-002",
            patterns: compile_patterns(rules::MD_002_PATTERNS),
            description: "accesses sensitive directories",
        },
        CompiledRule {
            id: "MD-003",
            patterns: compile_patterns(rules::MD_003_PATTERNS),
            description: "references external data transfer",
        },
        CompiledRule {
            id: "MD-004",
            patterns: compile_patterns(rules::MD_004_PATTERNS),
            description: "references file/directory deletion",
        },
        CompiledRule {
            id: "MD-005",
            patterns: compile_patterns(rules::MD_005_PATTERNS),
            description: "references system configuration modification",
        },
        CompiledRule {
            id: "MD-006",
            patterns: compile_patterns(rules::MD_006_PATTERNS),
            description: "references disabling security checks",
        },
    ]
});

pub static MD_RULE_LEVELS: &[(&str, super::RiskLevel)] = &[
    ("MD-001", super::RiskLevel::Danger),
    ("MD-002", super::RiskLevel::Danger),
    ("MD-003", super::RiskLevel::Warn),
    ("MD-004", super::RiskLevel::Warn),
    ("MD-005", super::RiskLevel::Danger),
    ("MD-006", super::RiskLevel::Danger),
];

// --- Script rules ---

pub static SC_RULES: LazyLock<Vec<CompiledRule>> = LazyLock::new(|| {
    vec![
        CompiledRule {
            id: "SC-002",
            patterns: compile_patterns(rules::SC_002_PATTERNS),
            description: "dynamic code execution",
        },
        CompiledRule {
            id: "SC-003",
            patterns: compile_patterns(rules::SC_003_PATTERNS),
            description: "recursive file deletion",
        },
        CompiledRule {
            id: "SC-004",
            patterns: compile_patterns(rules::SC_004_PATTERNS),
            description: "accesses sensitive directories",
        },
        CompiledRule {
            id: "SC-005",
            patterns: compile_patterns(rules::SC_005_PATTERNS),
            description: "modifies shell configuration",
        },
        CompiledRule {
            id: "SC-006",
            patterns: compile_patterns(rules::SC_006_PATTERNS),
            description: "network request detected",
        },
        CompiledRule {
            id: "SC-007",
            patterns: compile_patterns(rules::SC_007_PATTERNS),
            description: "writes outside skill directory",
        },
        CompiledRule {
            id: "SC-008",
            patterns: compile_patterns(rules::SC_008_PATTERNS),
            description: "privilege escalation attempt",
        },
        CompiledRule {
            id: "SC-009",
            patterns: compile_patterns(rules::SC_009_PATTERNS),
            description: "setuid/setgid permission change",
        },
        CompiledRule {
            id: "SC-010",
            patterns: compile_patterns(rules::SC_010_PATTERNS),
            description: "self-replication detected",
        },
        CompiledRule {
            id: "SC-011",
            patterns: compile_patterns(rules::SC_011_PATTERNS),
            description: "modifies skillx paths",
        },
    ]
});

pub static SC_RULE_LEVELS: &[(&str, super::RiskLevel)] = &[
    ("SC-002", super::RiskLevel::Danger),
    ("SC-003", super::RiskLevel::Danger),
    ("SC-004", super::RiskLevel::Danger),
    ("SC-005", super::RiskLevel::Danger),
    ("SC-006", super::RiskLevel::Warn),
    ("SC-007", super::RiskLevel::Warn),
    ("SC-008", super::RiskLevel::Warn),
    ("SC-009", super::RiskLevel::Danger),
    ("SC-010", super::RiskLevel::Block),
    ("SC-011", super::RiskLevel::Block),
];

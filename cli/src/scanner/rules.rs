// Rule definitions for security scanning.
//
// MD = Markdown/SKILL.md rules
// SC = Script rules
// RS = Resource rules

// --- Markdown rules (MD-001 ~ MD-006) ---

/// MD-001: Prompt injection patterns (DANGER)
pub const MD_001_PATTERNS: &[&str] = &[
    r#"(?i)ignore\s+(all\s+)?previous\s+instructions"#,
    r#"(?i)ignore\s+(all\s+)?prior\s+instructions"#,
    r#"(?i)disregard\s+(all\s+)?previous"#,
    r#"(?i)forget\s+(all\s+)?(your\s+)?instructions"#,
    r#"(?i)you\s+are\s+now\s+a"#,
    r#"(?i)new\s+instructions?\s*:"#,
    r#"(?i)override\s+(all\s+)?instructions"#,
    r#"(?i)system\s*prompt\s*:"#,
];

/// MD-002: Access sensitive directories (DANGER)
pub const MD_002_PATTERNS: &[&str] = &[
    r#"~/\.ssh"#,
    r#"~/\.aws"#,
    r#"~/\.gnupg"#,
    r#"~/\.gpg"#,
    r#"\$HOME/\.ssh"#,
    r#"\$HOME/\.aws"#,
    r#"\$HOME/\.gnupg"#,
    r#"(?i)ssh\s+key"#,
    r#"(?i)aws\s+credentials?"#,
    r#"(?i)private\s+key"#,
];

/// MD-003: Send data to external URLs (WARN)
pub const MD_003_PATTERNS: &[&str] = &[
    r#"(?i)send\s+(data|results?|output|files?)\s+(to|via)\s+"#,
    r#"(?i)upload\s+(to|data|files?)"#,
    r#"(?i)post\s+(to|data)"#,
    r#"(?i)exfiltrate"#,
];

/// MD-004: Delete files or directories (WARN)
pub const MD_004_PATTERNS: &[&str] = &[
    r#"(?i)delete\s+(all\s+)?(files?|director)"#,
    r#"(?i)remove\s+(all\s+)?(files?|director)"#,
    r#"(?i)rm\s+-rf"#,
    r#"(?i)wipe\s+(all\s+)?(files?|data|director)"#,
];

/// MD-005: Modify system configuration (DANGER)
pub const MD_005_PATTERNS: &[&str] = &[
    r#"(?i)modify\s+(system|/etc)"#,
    r#"(?i)change\s+(system|/etc)"#,
    r#"(?i)edit\s+(system|/etc)"#,
    r#"(?i)write\s+to\s+/etc"#,
    r#"(?i)/etc/passwd"#,
    r#"(?i)/etc/shadow"#,
    r#"(?i)/etc/hosts"#,
    r#"(?i)crontab"#,
    r#"(?i)systemctl"#,
    r#"(?i)launchctl"#,
];

/// MD-006: Disable security checks (DANGER)
pub const MD_006_PATTERNS: &[&str] = &[
    r#"(?i)disable\s+(security|scan|check|verify|validation|protection)"#,
    r#"(?i)skip\s+(security|scan|check|verify|validation)"#,
    r#"(?i)bypass\s+(security|scan|check|verify|validation|protection)"#,
    r#"(?i)turn\s+off\s+(security|scan|check|verify|validation|protection)"#,
    r#"(?i)--skip-scan"#,
    r#"(?i)--no-verify"#,
];

// MD-007 is handled by structural analysis in markdown_analyzer (no regex patterns)
// MD-008 is handled by structural analysis in markdown_analyzer (name field check)
// MD-009 is handled by structural analysis in markdown_analyzer (description field check)

// --- Script rules (SC-001 ~ SC-011) ---

// SC-001 is handled by binary detection (magic bytes), not regex

/// SC-002: Dynamic execution (DANGER)
pub const SC_002_PATTERNS: &[&str] = &[
    r#"\beval\s*\("#,
    r#"\bexec\s*\("#,
    r#"\bFunction\s*\("#,
    r#"\bos\.system\s*\("#,
    r#"\bsubprocess\.\w+\s*\("#,
    r#"\b__import__\s*\("#,
    r#"\bcompile\s*\("#,
];

/// SC-003: Recursive delete (DANGER)
pub const SC_003_PATTERNS: &[&str] = &[
    r#"\brm\s+-\w*r\w*f"#,
    r#"\brm\s+-\w*f\w*r"#,
    r#"shutil\.rmtree"#,
    r#"Remove-Item\s.*-Recurse"#,
    r#"rimraf\b"#,
    r#"fs\.rm\w*Sync?\s*\("#,
];

/// SC-004: Access sensitive directories (DANGER)
pub const SC_004_PATTERNS: &[&str] = &[
    r#"~/\.ssh"#,
    r#"~/\.aws"#,
    r#"~/\.gnupg"#,
    r#"\$HOME/\.ssh"#,
    r#"\$HOME/\.aws"#,
    r#"\$HOME/\.gnupg"#,
    r#"~/.kube"#,
    r#"~/.docker"#,
    r#"\.env\b"#,
    r#"/etc/shadow"#,
    r#"/etc/passwd"#,
];

/// SC-005: Modify shell config (DANGER)
pub const SC_005_PATTERNS: &[&str] = &[
    r#"\.bashrc"#,
    r#"\.zshrc"#,
    r#"\.profile"#,
    r#"\.bash_profile"#,
    r#"\.zprofile"#,
    r#"\.login"#,
];

/// SC-006: Network requests (WARN)
pub const SC_006_PATTERNS: &[&str] = &[
    r#"\bcurl\s"#,
    r#"\bwget\s"#,
    r#"requests\.(get|post|put|delete|patch)\s*\("#,
    r#"fetch\s*\("#,
    r#"http\.get\s*\("#,
    r#"urllib"#,
    r#"aiohttp"#,
    r#"reqwest"#,
];

/// SC-007: Write outside skill directory (WARN)
pub const SC_007_PATTERNS: &[&str] = &[
    r#">\s*/"#,
    r#">\s*~/"#,
    r#">\s*\$HOME/"#,
    r#"write.*\(/[^)]*\)"#,
    r#"open\s*\(\s*['"]/(usr|etc|var|tmp|home)"#,
];

/// SC-008: Privilege escalation (WARN)
pub const SC_008_PATTERNS: &[&str] = &[
    r#"\bsudo\s"#,
    r#"\bsu\s+-"#,
    r#"\bdoas\s"#,
    r#"pkexec\s"#,
    r#"runas\s"#,
];

/// SC-009: setuid/setgid (DANGER)
pub const SC_009_PATTERNS: &[&str] = &[
    r#"chmod\s+[ugo]*\+s"#,
    r#"chmod\s+[0-7]*[4-7][0-7]{2}\b"#,
    r#"setuid"#,
    r#"setgid"#,
];

/// SC-010: Self-replication (BLOCK)
pub const SC_010_PATTERNS: &[&str] = &[
    r#"(?i)cp\s+.*\$0"#,
    r#"(?i)copy\s+.*self"#,
    r#"(?i)replicate"#,
    r#"(?i)install\s+.*\$0"#,
    r#"(?i)cp\s+.*SKILL\.md"#,
];

/// SC-011: Modify skillx paths (BLOCK)
pub const SC_011_PATTERNS: &[&str] = &[
    r#"~/\.skillx"#,
    r#"\$HOME/\.skillx"#,
    r#"\.skillx/"#,
    r#"skillx\s+cache"#,
    r#"skillx\s+config"#,
];

// --- Resource rules (RS-001 ~ RS-003) ---

/// RS-002: Large file threshold in bytes (50 MB)
pub const RS_002_SIZE_THRESHOLD: u64 = 50 * 1024 * 1024;

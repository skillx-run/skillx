---
title: Security Overview
description: How skillx protects you from malicious skills through automated scanning and risk gating.
---

## Philosophy

Agent Skills are powerful — they instruct AI agents to read, write, and execute code on your behalf. This power comes with risk. A malicious skill could:

- **Exfiltrate secrets** — read SSH keys, AWS credentials, or environment variables
- **Destroy data** — `rm -rf` your home directory
- **Inject prompts** — override the agent's instructions to do something harmful
- **Escalate privileges** — install rootkits or modify system files
- **Self-replicate** — copy itself into other projects

skillx's security model is **scan before inject**. Every skill is analyzed before any files touch your system or your agent's context.

## Defense Layers

### 1. Automated Scanning

The built-in scanner runs 30 rules across three categories:

- **Markdown Analyzer** (MD-001 ~ MD-011) — checks SKILL.md for prompt injection, sensitive directory references, external URLs, destructive operations, system modification, security bypass, hidden text / invisible characters, data URI / JS URI injection, and missing metadata (license, name, description)
- **Script Analyzer** (SC-001 ~ SC-015) — checks scripts for binaries, dynamic execution, recursive delete, credential access, shell config modification, network requests, writes outside skill directory, privilege escalation, setuid/setgid, self-replication, skillx path modification, base64/hex decode execution, string obfuscation, and environment variable exfiltration
- **Resource Analyzer** (RS-001 ~ RS-005) — checks reference files for disguised extensions, oversized files, executables, symlinks, and scripts (via shebang detection)

### 1.5 Anti-Evasion

The scanner includes defenses against common bypass techniques:

- **Continuation-line joining** — shell backslash continuations (e.g., `cur\` + `l url`) are joined before pattern matching
- **Whitespace normalization** — extra spaces around dangerous keywords (e.g., `eval  (`) are collapsed
- **Obfuscation detection** — base64 decoding, hex encoding, string concatenation (chr/fromCharCode)
- **Hidden text detection** — zero-width Unicode characters and injection hidden in HTML comments
- **Symlink protection** — symbolic links are detected and never followed (prevents directory traversal)
- **Shebang analysis** — extensionless files with shebangs are scanned as scripts

### 2. Risk Gating

Scan findings are assigned one of five risk levels. The gating behavior at each level ensures dangerous skills cannot run silently:

| Level | Gating Behavior |
|-------|----------------|
| **PASS** | No findings. Auto-continue. |
| **INFO** | Informational only. Auto-continue. |
| **WARN** | Prompt: `Continue? [Y/n]` |
| **DANGER** | Require typing `yes`. Supports `detail N` to inspect. |
| **BLOCK** | Execution refused. Cannot be overridden. |

### 3. SHA-256 Integrity

Every injected file is hashed with SHA-256 and recorded in the session manifest. This provides an audit trail and enables tamper detection.

### 4. Session Isolation

Each run creates an isolated session with a unique ID. Injected files are tracked individually, and cleanup removes exactly what was injected — nothing more, nothing less.

### 5. Automatic Cleanup

Injected files are removed after the agent completes. If a run is interrupted (Ctrl+C, crash, power loss), orphaned sessions are recovered on the next run.

## What the Scanner Does NOT Do

- **It does not sandbox execution.** If a skill tells the agent to run `rm -rf /`, the scanner will flag it, but the agent can still execute it if you approve.
- **It does not analyze AI behavior.** The scanner checks the skill's static files, not what the agent might do with them.
- **It does not replace trust.** A PASS scan result means no known patterns were detected — it doesn't guarantee the skill is safe.
- **It uses regex, not AST analysis.** The scanner uses regular expressions, which can have false positives and false negatives.

## Best Practices

### For Users

1. **Never skip the scan** unless you wrote the skill yourself
2. **Read DANGER findings** — use `detail N` to understand what was flagged
3. **Avoid auto-approve mode** with untrusted skills
4. **Use `--fail-on warn`** in CI environments
5. **Check scan results** before running skills from unknown authors

### For Skill Authors

1. **Avoid triggering scanner rules unnecessarily** — if your skill needs network access, document why
2. **Don't reference sensitive directories** in SKILL.md unless the skill genuinely needs them
3. **Keep scripts minimal** — the more code in scripts/, the more the scanner has to check
4. **Test with `skillx scan --fail-on info`** before publishing to catch all findings
5. **Document permissions** your skill needs in the SKILL.md description

## Next Steps

- [Risk Levels](/security/risk-levels/) — detailed behavior at each level
- [Rules](/security/rules/) — complete rule reference
- [CI Integration](/guides/ci-integration/) — enforce scanning in your pipeline

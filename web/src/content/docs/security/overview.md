---
title: Security Overview
description: Trust model and safety boundaries for running untrusted skills with skillx.
---

## Trust Model

skillx is built around a simple rule:

- **Scan before inject**
- **Gate before launch**
- **Clean after exit**

That sequence is the core trust model for skillx. You should be able to inspect a skill before it reaches your agent, approve risk before execution starts, and trust that temporary files are removed when the run ends.

This matters because Agent Skills can ask an AI agent to read files, write code, run shell commands, and fetch remote content on your behalf. A malicious skill could try to exfiltrate secrets, destroy data, inject instructions, escalate privileges, or persist beyond the session. skillx reduces that risk by adding explicit control points around the full lifecycle.

## What skillx protects

skillx is designed to protect three boundaries:

- **Your machine**: reduce the chance that untrusted skill files touch sensitive locations or ship obvious destructive behavior.
- **Your agent context**: surface prompt injection and hidden instructions before they are copied into the agent's working environment.
- **Your session lifecycle**: keep temporary injection isolated, auditable, and removable after the run.

## How the trust flow works

### 1. Scan before inject

Before any skill files are injected into an agent environment, skillx analyzes the source with built-in scanners. The scanner runs 30 rules across three analyzers:

- **Markdown Analyzer** (`MD-001` to `MD-011`) checks `SKILL.md` for prompt injection, sensitive directory references, destructive instructions, hidden text, external URLs, and missing metadata.
- **Script Analyzer** (`SC-001` to `SC-015`) checks helper scripts for dynamic execution, recursive deletes, credential access, shell profile changes, privilege escalation, self-replication, and obfuscation.
- **Resource Analyzer** (`RS-001` to `RS-005`) checks references and assets for disguised extensions, oversized files, executables, symlinks, and script payloads.

The scanner also includes anti-evasion checks such as continuation-line joining, whitespace normalization, hidden text detection, obfuscation detection, shebang analysis, and symlink protection. The goal is not to prove safety, but to catch risky patterns before they reach the agent.

### 2. Gate before launch

Findings are turned into explicit risk levels, and those levels control whether a run can proceed:

| Level | Gating Behavior |
|-------|----------------|
| **PASS** | No findings. Auto-continue. |
| **INFO** | Informational only. Auto-continue. |
| **WARN** | Prompt: `Continue? [Y/n]` |
| **DANGER** | Require typing `yes`. Supports `detail N` to inspect. |
| **BLOCK** | Execution refused. Cannot be overridden. |

This is the second trust boundary: skillx does not silently continue from a risky scan. You see the result before launch, and truly blocked content never runs.

### 3. Clean after exit

Each run gets its own session ID and manifest. Injected files are hashed with SHA-256, tracked individually, and removed when the agent finishes. If the process is interrupted, orphaned sessions are recovered on the next run.

This keeps temporary skill files from becoming hidden long-term state on your system.

## Defense Layers

### Automated scanning

The scanner is the first control point. It looks for risky instructions in Markdown, scripts, and bundled resources before the skill is injected anywhere.

### Risk gating

The gate is the second control point. It converts scan output into an approval workflow so users decide whether a risky skill should proceed.

### Session integrity

SHA-256 manifests provide an audit trail for what was injected during a run and make cleanup deterministic.

### Session isolation and recovery

skillx isolates each run, removes exactly the files it injected, and recovers interrupted sessions on the next invocation.

## What the Scanner Does NOT Do

- **It does not sandbox execution.** If a skill tells the agent to run `rm -rf /`, the scanner will flag it, but the agent can still execute it if you approve.
- **It does not analyze AI behavior.** The scanner checks the skill's static files, not what the agent might do with them.
- **It does not replace trust.** A PASS scan result means no known patterns were detected — it doesn't guarantee the skill is safe.
- **It uses regex, not AST analysis.** The scanner uses regular expressions, which can have false positives and false negatives.

## Best Practices

### For Users

1. **Treat PASS as a trust signal, not a guarantee**
2. **Read DANGER findings** with `detail N` before approving
3. **Avoid auto-approve mode** for untrusted or newly discovered skills
4. **Use `--fail-on warn`** in CI or policy-driven environments
5. **Prefer pinned refs** when running remote skills from Git hosts

### For Skill Authors

1. **Document why elevated behavior is necessary** if your skill needs network, shell, or filesystem access
2. **Avoid referencing sensitive directories** unless the skill genuinely requires them
3. **Keep helper scripts minimal and readable**
4. **Test with `skillx scan --fail-on info`** before publishing
5. **Write SKILL.md for auditability** so users can understand intent quickly

## Next Steps

- [Risk Levels](/security/risk-levels/) for the exact approval behavior at each level
- [Rules](/security/rules/) for the complete scanner rule reference
- [CI Integration](/guides/ci-integration/) for enforcing scan policy in automation

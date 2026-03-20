---
title: "skillx scan"
description: Reference for the skillx scan command — security scan a skill and report findings.
---

## Synopsis

```bash
skillx scan <source> [options]
```

Run the security scanner against a skill without executing it. Useful for auditing skills before use or integrating into CI pipelines.

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `source` | Yes | Skill source: local path, `github:` prefix, or GitHub URL |

## Options

| Flag | Default | Description |
|------|---------|-------------|
| `--format <fmt>` | `text` | Output format: `text` or `json` |
| `--fail-on <level>` | `danger` | Exit with code 1 if any finding meets or exceeds this level |

## Output Formats

### Text (default)

Human-readable output on stderr:

```
  DANGER  MD-001  SKILL.md:7   Prompt injection pattern detected
  WARN    MD-003  SKILL.md:12  References external URL
  INFO    SC-006  scripts/fetch.sh:3  Network request detected (curl)
```

For a clean skill:

```
  PASS — no findings
```

### JSON

Machine-readable output on stdout:

```bash
skillx scan --format json ./my-skill
```

```json
{
  "findings": [
    {
      "rule_id": "MD-001",
      "level": "danger",
      "message": "Prompt injection pattern detected",
      "file": "SKILL.md",
      "line": 7,
      "context": null
    }
  ]
}
```

JSON output goes to stdout so it can be piped to `jq` or other tools. Status messages still go to stderr.

## Fail Threshold

The `--fail-on` flag sets the minimum risk level that causes a non-zero exit code:

| `--fail-on` | Exits 1 when overall level is... |
|-------------|----------------------------------|
| `info` | INFO or higher |
| `warn` | WARN or higher |
| `danger` (default) | DANGER or higher |
| `block` | BLOCK only |

The overall level is the maximum of all individual findings.

```bash
# Strict: fail on any warning
skillx scan --fail-on warn ./my-skill

# Lenient: only fail on block
skillx scan --fail-on block ./my-skill
```

## What Gets Scanned

The scanner checks three categories of files:

### SKILL.md (Markdown Analyzer)

Rules MD-001 through MD-006 check the main instruction file for:

- Prompt injection patterns
- References to sensitive directories
- External URL references
- Destructive file operations
- System configuration modification
- Security bypass instructions

### scripts/ (Script Analyzer)

Rules SC-001 through SC-011 check all scripts for:

- Embedded binaries (magic byte detection)
- Dynamic execution (`eval`, `exec`, `subprocess`)
- Recursive delete operations
- Credential directory access
- Shell config modification
- Network requests
- Writes outside the skill directory
- Privilege escalation
- Setuid/setgid operations
- Self-replication patterns
- Modification of skillx paths

### references/ (Resource Analyzer)

Rules RS-001 through RS-003 check reference files for:

- Disguised file extensions
- Oversized files (> 50 MB)
- Executable content in reference files

Root-level script files (`.py`, `.sh`, `.js`, etc.) are also scanned with the Script Analyzer.

## CI Integration

```bash
# In a GitHub Actions workflow
- name: Scan skill
  run: |
    skillx scan --format json --fail-on warn ./my-skill > scan-report.json

# In a shell script
if ! skillx scan --fail-on warn ./my-skill; then
  echo "Skill failed security scan"
  exit 1
fi
```

See [CI Integration guide](/guides/ci-integration/) for more detailed examples.

## Examples

### Scan a local skill

```bash
skillx scan ./my-skill
```

### Scan a GitHub skill

```bash
skillx scan github:org/skills/data-pipeline
```

### Generate JSON report

```bash
skillx scan --format json ./my-skill | jq '.findings[] | select(.level == "danger")'
```

### Strict scan for CI

```bash
skillx scan --fail-on warn --format json ./my-skill
```

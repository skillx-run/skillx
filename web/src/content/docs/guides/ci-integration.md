---
title: CI Integration
description: Using skillx scan in CI pipelines to enforce security standards for Agent Skills.
---

## Overview

`skillx scan` is designed for CI/CD integration. It can:

- Scan skills on every pull request
- Enforce a minimum security threshold
- Produce machine-readable JSON reports
- Exit with non-zero codes when thresholds are exceeded

## GitHub Actions

### Official Action (Recommended)

The simplest way to integrate skillx into GitHub Actions. Automatically installs skillx, runs the scan, and uploads SARIF results to the GitHub Security tab.

```yaml
name: Skill Security Scan
on:
  pull_request:
    paths:
      - 'skills/**'

permissions:
  contents: read
  security-events: write

jobs:
  scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Scan skill
        uses: skillx-run/skillx/.github/actions/scan@main
        with:
          source: ./my-skill
          fail-on: warn
```

#### Inputs

| Input | Required | Default | Description |
|-------|----------|---------|-------------|
| `source` | Yes | — | Skill source to scan (local path or URL) |
| `fail-on` | No | `danger` | Risk level threshold (info, warn, danger, block) |
| `format` | No | `sarif` | Output format (text, json, sarif) |
| `upload-sarif` | No | `true` | Upload SARIF to GitHub Code Scanning |

#### Outputs

| Output | Description |
|--------|-------------|
| `sarif-file` | Path to the SARIF output file |
| `level` | Highest risk level found |
| `findings-count` | Number of findings |

### Multiple Skills with Matrix

```yaml
name: Skill Security Scan
on: [pull_request]

permissions:
  contents: read
  security-events: write

jobs:
  scan:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        skill:
          - skills/pdf-processing
          - skills/code-review
          - skills/data-pipeline
    steps:
      - uses: actions/checkout@v4

      - name: Scan ${{ matrix.skill }}
        uses: skillx-run/skillx/.github/actions/scan@main
        with:
          source: ${{ matrix.skill }}
          fail-on: warn
```

### Manual Installation

If you prefer to install skillx manually instead of using the official action:

```yaml
      - name: Install skillx
        run: curl -fsSL https://skillx.run/install.sh | sh

      - name: Scan skill
        run: skillx scan --format sarif --fail-on warn ./my-skill > results.sarif

      - name: Upload SARIF
        if: always()
        uses: github/codeql-action/upload-sarif@v3
        with:
          sarif_file: results.sarif
          category: skillx-scan
```

### With JSON Report

```yaml
      - name: Install skillx
        run: curl -fsSL https://skillx.run/install.sh | sh

      - name: Scan skill
        run: |
          skillx scan --format json --fail-on warn ./my-skill > scan-report.json
          cat scan-report.json

      - name: Upload scan report
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: scan-report
          path: scan-report.json
```

## GitLab CI

```yaml
skill-scan:
  image: ubuntu:latest
  stage: test
  before_script:
    - apt-get update && apt-get install -y curl
    - curl -fsSL https://skillx.run/install.sh | sh
    - export PATH="$HOME/.local/bin:$PATH"
  script:
    - skillx scan --format json --fail-on warn ./my-skill > scan-report.json
  artifacts:
    when: always
    paths:
      - scan-report.json
  only:
    changes:
      - my-skill/**
```

## Shell Script

For any CI system, a simple shell script works:

```bash
#!/usr/bin/env bash
set -euo pipefail

FAIL_ON="${FAIL_ON:-warn}"
SKILLS_DIR="${SKILLS_DIR:-./skills}"

echo "Scanning skills in $SKILLS_DIR (fail-on: $FAIL_ON)..."

failed=0
for skill_dir in "$SKILLS_DIR"/*/; do
    if [ ! -f "$skill_dir/SKILL.md" ]; then
        continue
    fi

    echo ""
    echo "=== Scanning: $skill_dir ==="

    if ! skillx scan --fail-on "$FAIL_ON" "$skill_dir"; then
        echo "FAILED: $skill_dir"
        failed=1
    fi
done

if [ "$failed" -eq 1 ]; then
    echo ""
    echo "One or more skills failed the security scan."
    exit 1
fi

echo ""
echo "All skills passed the security scan."
```

## Pre-commit Hook

Scan skills before committing:

```bash
#!/usr/bin/env bash
# .git/hooks/pre-commit

# Find modified SKILL.md files and scan their parent directories
git diff --cached --name-only | grep 'SKILL.md' | while read -r file; do
    dir=$(dirname "$file")
    echo "Scanning $dir..."
    if ! skillx scan --fail-on warn "$dir"; then
        echo "Skill scan failed for $dir. Commit blocked."
        exit 1
    fi
done
```

Make it executable:

```bash
chmod +x .git/hooks/pre-commit
```

## JSON Report Format

The `--format json` output follows this structure:

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
    },
    {
      "rule_id": "SC-006",
      "level": "warn",
      "message": "Network request detected (curl)",
      "file": "scripts/fetch.sh",
      "line": 3,
      "context": null
    }
  ]
}
```

### Processing with jq

```bash
# Count findings by level
skillx scan --format json ./my-skill | jq '.findings | group_by(.level) | map({level: .[0].level, count: length})'

# List only DANGER findings
skillx scan --format json ./my-skill | jq '.findings[] | select(.level == "danger")'

# Check if any BLOCK findings exist
skillx scan --format json ./my-skill | jq -e '.findings | map(select(.level == "block")) | length > 0'
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Scan passed (no findings at or above `--fail-on` threshold) |
| 1 | Scan failed (findings at or above `--fail-on` threshold) |

## Recommended Thresholds

| Environment | `--fail-on` | Rationale |
|-------------|-------------|-----------|
| Development | `danger` | Allow warnings during development |
| Pull request | `warn` | Catch potential issues before merge |
| Release | `warn` | Strict enforcement for published skills |
| Public registry | `info` | Maximum strictness for community skills |

## Headless Mode

When using `skillx run` (not just `scan`) in CI, the interactive scan gate prompts need to be disabled. Use `--headless` mode:

```bash
skillx run --headless --fail-on warn ./skill "prompt" --agent universal -p
```

In headless mode:
- **PASS/INFO/WARN** → auto-pass (no prompt)
- **DANGER/BLOCK** → auto-refuse with non-zero exit code

### Auto-Detection

The `CI=true` environment variable is auto-detected — most CI systems (GitHub Actions, GitLab CI, CircleCI, etc.) set this automatically. You can also set `SKILLX_HEADLESS=1` explicitly.

### Config Default

To make headless mode the default for a team:

```toml
# ~/.skillx/config.toml
[scan]
headless = true
```

## Tips

- **Cache the skillx binary** to avoid rebuilding on every CI run
- **Use `--format json`** for programmatic processing of results
- **Pin to a specific version** of skillx in CI to avoid unexpected behavior from updates
- **Run scans in parallel** for repositories with many skills
- **Store scan reports as artifacts** for debugging failed scans

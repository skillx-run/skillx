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

### Basic Scan

```yaml
name: Skill Security Scan
on:
  pull_request:
    paths:
      - 'skills/**'

jobs:
  scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install skillx
        run: cargo install skillx

      - name: Scan all skills
        run: |
          for dir in skills/*/; do
            echo "Scanning $dir..."
            skillx scan --fail-on warn "$dir"
          done
```

### With JSON Report

```yaml
name: Skill Security Scan
on: [pull_request]

jobs:
  scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install skillx
        run: cargo install skillx

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

### Matrix Strategy for Multiple Skills

```yaml
name: Skill Security Scan
on: [pull_request]

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

      - name: Install skillx
        run: cargo install skillx

      - name: Scan ${{ matrix.skill }}
        run: skillx scan --fail-on warn ${{ matrix.skill }}
```

### Cache Cargo Build

Speed up CI by caching the Cargo build:

```yaml
      - name: Cache cargo
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/bin/
            ~/.cargo/registry/
            ~/.cargo/git/
          key: ${{ runner.os }}-cargo-skillx

      - name: Install skillx
        run: |
          if ! command -v skillx &> /dev/null; then
            cargo install skillx
          fi
```

## GitLab CI

```yaml
skill-scan:
  image: rust:latest
  stage: test
  before_script:
    - cargo install skillx
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

## Tips

- **Cache the skillx binary** to avoid rebuilding on every CI run
- **Use `--format json`** for programmatic processing of results
- **Pin to a specific version** of skillx in CI to avoid unexpected behavior from updates
- **Run scans in parallel** for repositories with many skills
- **Store scan reports as artifacts** for debugging failed scans

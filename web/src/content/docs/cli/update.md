---
title: "skillx update"
description: Reference for the skillx update command — update installed skills to their latest versions.
---

## Synopsis

```bash
skillx update [names...] [options]
```

Update installed skills to their latest versions. Uses SHA-256 content comparison for precise change detection.

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `names` | No | Skill name(s) to update. If omitted, checks all installed skills |

## Options

| Flag | Default | Description |
|------|---------|-------------|
| `--agent <name>` | — | Only update for a specific agent |
| `--dry-run` | — | Show what would be updated without applying changes |
| `--skip-scan` | — | Skip security scan for updated content |
| `--yes` | — | Auto-confirm (skip the update prompt) |

## Update Process

### 1. Fetch Latest

For each installed skill, skillx fetches the latest content from the original source. Local sources are skipped during update checks.

### 2. Compare

Each file is compared using SHA-256 hashes. Only skills with actual content changes are flagged for update.

### 3. Summary

A summary table is displayed:

```
Updates Available

  Name              Installed   Available   Changes
  pdf-processing    v1.2        v1.3        2 files changed
  formatter         v2.1        v2.2        1 file changed

Update 2 skill(s)? [Y/n]
```

### 4. Scan

Unless `--skip-scan` is specified, updated content is scanned through the security gate before applying.

### 5. Apply

Updated files are written to all agent injection paths. Progress is saved after each successful skill update, so a mid-update failure does not lose already-applied updates.

### 6. Sync

Updated source strings (including new version refs) are synced back to skillx.toml.

## Dry Run

Use `--dry-run` to see what would be updated without making any changes:

```bash
skillx update --dry-run
```

This fetches and compares all skills but does not apply changes, modify installed.json, or update skillx.toml.

## Examples

### Update all installed skills

```bash
skillx update
```

### Update specific skills

```bash
skillx update pdf-processing formatter
```

### Check what would be updated

```bash
skillx update --dry-run
```

### Update without prompts

```bash
skillx update --yes
```

### Update only for a specific agent

```bash
skillx update --agent claude-code
```

## Related Docs

- [Manage Project Skills](/guides/manage-project-skills/) for the full project-skill lifecycle and recommended command order
- [skillx list](/cli/list/) to inspect installed versions before or after an update
- [skillx scan](/cli/scan/) if you want an explicit security review workflow alongside updates

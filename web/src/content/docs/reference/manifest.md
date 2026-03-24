---
title: manifest.json Reference
description: Complete reference for the session manifest format used by skillx.
---

## Overview

Every `skillx run` creates a session manifest — a JSON file that tracks exactly what was injected, where, and when. Manifests serve two purposes:

1. **Cleanup** — knowing exactly which files to remove when the session ends
2. **Audit trail** — recording what happened for later review

## Location

During execution:
```
~/.skillx/active/<session-id>/manifest.json
```

After cleanup (archived):
```
~/.skillx/history/<session-id>.json
```

## Schema

```json
{
  "session_id": "a1b2c3d4",
  "skill_name": "pdf-processing",
  "source": "github:skillx-run/skillx.run/examples/skills/code-review",
  "agent": "claude-code",
  "lifecycle_mode": "ManagedProcess",
  "scope": "global",
  "created_at": "2025-03-15T14:30:00Z",
  "injected_files": [
    {
      "path": "/Users/me/.claude/skills/pdf-processing/SKILL.md",
      "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    },
    {
      "path": "/Users/me/.claude/skills/pdf-processing/scripts/extract.py",
      "sha256": "d7a8fbb307d7809469ca9abcb0082e4f8d5651e46d3cdb762d02d0bf37c9e592"
    }
  ],
  "injected_attachments": [
    {
      "original": "./report.pdf",
      "copied_to": "/Users/me/.claude/skills/pdf-processing/attachments/report.pdf"
    }
  ],
  "scan_result": {
    "findings": [
      {
        "rule_id": "SC-006",
        "level": "warn",
        "message": "Network request detected (curl)",
        "file": "scripts/extract.py",
        "line": 42,
        "context": null
      }
    ]
  }
}
```

## Fields

### Top-Level

| Field | Type | Description |
|-------|------|-------------|
| `session_id` | string | Unique 8-character ID (truncated UUID v4) |
| `skill_name` | string | Name from SKILL.md frontmatter or directory name |
| `source` | string | Original source string passed to `skillx run` |
| `agent` | string | Agent adapter name (e.g., `claude-code`) |
| `lifecycle_mode` | string | `ManagedProcess` or `FileInjectAndWait` |
| `scope` | string | `global` or `project` |
| `created_at` | string | ISO 8601 timestamp (UTC) |
| `injected_files` | array | Files copied to the agent's skill directory |
| `injected_attachments` | array | Extra files attached via `--attach` |
| `scan_result` | object/null | Security scan report (null if `--skip-scan`) |

### injected_files[]

| Field | Type | Description |
|-------|------|-------------|
| `path` | string | Absolute path where the file was written |
| `sha256` | string | SHA-256 hash of the file content (hex-encoded) |

### injected_attachments[]

| Field | Type | Description |
|-------|------|-------------|
| `original` | string | Original path passed to `--attach` |
| `copied_to` | string | Absolute path where the attachment was copied |

### scan_result

The `scan_result` field contains the full scan report or `null` if scanning was skipped.

| Field | Type | Description |
|-------|------|-------------|
| `findings` | array | List of findings from the security scanner |

Each finding:

| Field | Type | Description |
|-------|------|-------------|
| `rule_id` | string | Rule identifier (e.g., `MD-001`, `SC-006`) |
| `level` | string | Risk level: `pass`, `info`, `warn`, `danger`, `block` |
| `message` | string | Human-readable description |
| `file` | string | Relative path within the skill directory |
| `line` | number/null | Line number where the finding was detected |
| `context` | string/null | Additional context (reserved for future use) |

## Session Lifecycle

### Creation

When `skillx run` starts:

1. A new session ID is generated (8-char UUID v4 prefix)
2. The session directory is created: `~/.skillx/active/<id>/`
3. Subdirectories `skill-files/` and `attachments/` are created
4. The manifest is populated during the inject phase
5. The manifest is saved to `~/.skillx/active/<id>/manifest.json`

### Active

During execution, the manifest exists in `~/.skillx/active/<id>/`. This signals to future skillx runs that a session is active. If skillx detects stale sessions on startup, it recovers them.

### Cleanup

When the session ends (normally, Ctrl+C, or timeout):

1. Each file in `injected_files` is deleted
2. Each file in `injected_attachments` is deleted
3. Empty parent directories are cleaned up
4. The manifest is moved to `~/.skillx/history/<id>.json`
5. The session directory in `active/` is removed

### Orphan Recovery

If a previous run was interrupted before cleanup (e.g., power loss, `kill -9`), the next `skillx run` detects orphaned sessions in `~/.skillx/active/` and recovers them:

```
● Recovered 1 orphaned session(s)
```

## Reading Manifests

### List Recent Sessions

```bash
ls ~/.skillx/history/
```

### Inspect a Manifest

```bash
cat ~/.skillx/history/a1b2c3d4.json | jq '.'
```

### Find Sessions by Skill

```bash
cat ~/.skillx/history/*.json | jq 'select(.skill_name == "pdf-processing")'
```

### Check Integrity

Compare the SHA-256 hash in the manifest with the actual file hash:

```bash
# During an active session
SESSION_ID="a1b2c3d4"
MANIFEST="$HOME/.skillx/active/$SESSION_ID/manifest.json"

cat "$MANIFEST" | jq -r '.injected_files[] | "\(.sha256)  \(.path)"' | sha256sum -c
```

## History Retention

The number of archived manifests is controlled by `[history].max_entries` in `config.toml`:

```toml
[history]
max_entries = 50  # default
```

When the count exceeds this limit, the oldest archived manifests are removed.

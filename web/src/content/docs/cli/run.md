---
title: "skillx run"
description: Full reference for the skillx run command — fetch, scan, inject, run, and clean a skill in one step.
---

## Synopsis

```bash
skillx run <source> [prompt] [options]
```

Fetch a skill, scan it for security issues, inject it into the active agent's context, launch the agent, wait for completion, and clean up.

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `source` | Yes | Skill source: local path, `github:` prefix, or GitHub URL |
| `prompt` | No | Prompt text to pass to the agent |

## Options

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--file <path>` | `-f` | — | Read the prompt from a file |
| `--stdin` | — | — | Read the prompt from stdin |
| `--agent <name>` | — | auto-detect | Target agent (skip auto-detection) |
| `--scope <scope>` | — | `global` | Injection scope: `global` or `project` |
| `--attach <path>` | — | — | Attach files for the agent (repeatable) |
| `--no-cache` | — | — | Force re-fetch, skip cache |
| `--skip-scan` | — | — | Skip the security scan (not recommended) |
| `--yes` | — | — | Auto-confirm WARN level risks |
| `--yolo` | — | — | Pass permission-skip flags to the agent |
| `--timeout <dur>` | — | — | Maximum run duration (e.g., `30m`, `2h`) |

## Lifecycle Phases

### Phase 1: Resolve

The source string is resolved in this priority order:

1. **Local path** — starts with `./`, `/`, `~/`, or exists on disk
2. **`github:` prefix** — e.g., `github:owner/repo/path`
3. **GitHub URL** — e.g., `https://github.com/owner/repo/tree/main/path`

Remote skills are cached after download. Use `--no-cache` to force a fresh fetch.

### Phase 2: Scan

Unless `--skip-scan` is set, the skill is scanned by the built-in security engine. The scan report is printed to stderr.

### Phase 3: Gate

The overall risk level determines how the run proceeds:

| Level | Behavior |
|-------|----------|
| PASS / INFO | Auto-continue |
| WARN | Prompt `Continue? [Y/n]` (skipped with `--yes`) |
| DANGER | Require typing `yes`; supports `detail N` to inspect findings |
| BLOCK | Execution refused, exit code 1 |

### Phase 4: Detect Agent

If `--agent` is not specified, skillx auto-detects installed agents. If multiple are found, an interactive selector is shown.

Available agent names: `claude-code`, `codex`, `copilot`, `cursor`, `universal`.

### Phase 5: Inject

Skill files are copied to the agent's expected directory:

| Agent | Global Scope | Project Scope |
|-------|-------------|---------------|
| Claude Code | `~/.claude/skills/<name>/` | `.claude/skills/<name>/` |
| Codex | `~/.codex/skills/<name>/` | `.agents/skills/<name>/` |
| Copilot | `~/.github/skills/<name>/` | `.github/skills/<name>/` |
| Cursor | `~/.cursor/skills/<name>/` | `.cursor/skills/<name>/` |
| Universal | `~/.agents/skills/<name>/` | `.agents/skills/<name>/` |

Each file is hashed with SHA-256 and recorded in the session manifest.

### Phase 6: Launch

- **CLI agents** (Claude Code, Codex): spawned as child processes with the prompt as an argument
- **IDE agents** (Copilot, Cursor, Universal): prompt copied to clipboard; skillx waits for Enter

### Phase 7: Wait

skillx waits for the agent to finish. Supports:

- **Ctrl+C** — kills the agent process and triggers cleanup
- **`--timeout`** — kills the agent after the specified duration

### Phase 8: Clean

All injected files are removed, the session manifest is archived to `~/.skillx/history/`, and orphaned sessions from previous interrupted runs are recovered.

## Prompt Sources

Prompts are resolved in priority order:

1. **CLI argument**: `skillx run ./skill "Do the thing"`
2. **File**: `skillx run ./skill -f prompt.txt`
3. **Stdin**: `echo "Do the thing" | skillx run ./skill --stdin`
4. **None**: Agent launches without an initial prompt

## Examples

### Basic usage

```bash
skillx run ./my-skill "Refactor the auth module"
```

### GitHub skill with timeout

```bash
skillx run --timeout 30m github:org/skills/formatter "Format all Python files"
```

### Pipe prompt from another command

```bash
git diff HEAD~1 | skillx run ./code-review-skill --stdin
```

### Project-scoped injection

```bash
skillx run --scope project ./my-skill "Set up the project"
```

### YOLO mode with auto-confirm

```bash
skillx run --yes --yolo github:org/skills/migration "Run database migration"
```

### Attach context files

```bash
skillx run ./analysis-skill \
  --attach ./data.csv \
  --attach ./schema.sql \
  "Analyze the data and suggest optimizations"
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Error (scan blocked, agent failure, source resolution failed) |

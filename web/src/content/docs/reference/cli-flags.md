---
title: CLI Flags Reference
description: Complete flag reference for all skillx commands.
---

## Global

```bash
skillx [command] [options]
```

| Flag | Description |
|------|-------------|
| `--version` | Print version information |
| `--help` | Print help information |

## skillx run

```bash
skillx run <source> [prompt] [options]
```

### Arguments

| Argument | Position | Required | Description |
|----------|----------|----------|-------------|
| `source` | 1 | Yes | Skill source (local path, `github:` prefix, or GitHub URL) |
| `prompt` | 2 | No | Prompt text to pass to the agent |

### Flags

| Flag | Short | Type | Default | Description |
|------|-------|------|---------|-------------|
| `--file` | `-f` | string | — | Read prompt from a file path |
| `--stdin` | — | bool | false | Read prompt from stdin |
| `--agent` | — | string | auto-detect | Agent name: `claude-code`, `codex`, `copilot`, `cursor`, `universal` |
| `--scope` | — | string | `global` | Injection scope: `global` or `project` |
| `--attach` | — | string[] | — | Attach files for the agent (repeatable) |
| `--no-cache` | — | bool | false | Force re-fetch, skip cache |
| `--skip-scan` | — | bool | false | Skip the security scan |
| `--yes` | — | bool | false | Auto-confirm WARN level risks |
| `--yolo` | — | bool | false | Pass permission-skip flags to the agent |
| `--timeout` | — | string | — | Max run duration (e.g., `30m`, `2h`, `300s`) |

### Prompt Resolution Priority

1. Positional `prompt` argument
2. `--file` / `-f` flag
3. `--stdin` flag
4. No prompt (agent launches without one)

### Duration Format

The `--timeout` flag accepts human-friendly durations:

| Suffix | Unit | Example |
|--------|------|---------|
| `s` | Seconds | `300s` |
| `m` | Minutes | `30m` |
| `h` | Hours | `2h` |
| `d` | Days | `1d` |

### Examples

```bash
skillx run ./my-skill "prompt"
skillx run -f prompt.txt github:org/repo/skill
echo "prompt" | skillx run --stdin ./skill
skillx run --agent codex --yolo --timeout 1h ./skill "prompt"
skillx run --scope project --attach data.csv ./skill "analyze"
```

## skillx scan

```bash
skillx scan <source> [options]
```

### Arguments

| Argument | Position | Required | Description |
|----------|----------|----------|-------------|
| `source` | 1 | Yes | Skill source to scan |

### Flags

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--format` | string | `text` | Output format: `text` or `json` |
| `--fail-on` | string | `danger` | Exit code threshold: `pass`, `info`, `warn`, `danger`, `block` |

### Output Routing

- `text` format: output to **stderr**
- `json` format: output to **stdout** (pipeable)

### Examples

```bash
skillx scan ./my-skill
skillx scan --format json github:org/repo/skill
skillx scan --fail-on warn ./my-skill
skillx scan --format json --fail-on warn ./skill | jq '.'
```

## skillx agents

```bash
skillx agents [options]
```

### Flags

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--all` | bool | false | Show all known agents, including undetected |

### Examples

```bash
skillx agents
skillx agents --all
```

## skillx info

```bash
skillx info <source>
```

### Arguments

| Argument | Position | Required | Description |
|----------|----------|----------|-------------|
| `source` | 1 | Yes | Skill source to inspect |

Show metadata parsed from the skill's SKILL.md frontmatter.

### Examples

```bash
skillx info ./my-skill
skillx info github:org/repo/skill
```

## skillx cache

```bash
skillx cache <subcommand>
```

### Subcommands

| Subcommand | Description |
|------------|-------------|
| `ls` | List all cached skills with source and timestamp |
| `clean` | Remove all cached skill entries |

### Examples

```bash
skillx cache ls
skillx cache clean
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `GITHUB_TOKEN` | GitHub personal access token for private repositories |

## Exit Codes

| Code | Applies To | Meaning |
|------|-----------|---------|
| 0 | All commands | Success |
| 1 | All commands | Error or threshold exceeded |

For `skillx scan`, exit code 1 means findings were found at or above the `--fail-on` threshold. For `skillx run`, exit code 1 means the run could not complete (source resolution failed, scan blocked, agent error, etc.).

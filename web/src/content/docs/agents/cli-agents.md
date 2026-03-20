---
title: CLI Agents
description: How skillx works with Claude Code and OpenAI Codex — ManagedProcess lifecycle.
---

## Overview

CLI agents run as terminal processes that skillx spawns and manages directly. skillx passes the prompt as a CLI argument, waits for the process to exit, captures the exit code, and handles interruptions gracefully.

Both CLI agents use the **ManagedProcess** lifecycle mode.

## Claude Code

[Claude Code](https://github.com/anthropics/claude-code) is Anthropic's official CLI for Claude.

### Detection

skillx detects Claude Code by checking:

1. `claude` binary in PATH (via `which claude`)
2. `~/.claude/` directory exists

Either condition is sufficient.

### Injection Paths

| Scope | Path |
|-------|------|
| Global | `~/.claude/skills/<skill-name>/` |
| Project | `.claude/skills/<skill-name>/` |

### Launch

skillx spawns `claude` with the following arguments:

```bash
claude --prompt "your prompt here"
```

With YOLO mode (`--yolo`):

```bash
claude --prompt "your prompt here" --dangerously-skip-permissions
```

### YOLO Mode

Claude Code's `--dangerously-skip-permissions` flag skips all permission prompts. The agent can read, write, and execute without asking for confirmation.

```bash
skillx run --yolo github:org/skills/formatter "Format all files"
# Equivalent to: claude --prompt "..." --dangerously-skip-permissions
```

### Example Workflow

```bash
# Normal mode — Claude Code will ask for permissions
skillx run github:org/skills/code-review "Review the auth module"

# YOLO mode — no permission prompts
skillx run --yolo github:org/skills/formatter "Fix all lint errors"

# With timeout
skillx run --timeout 30m github:org/skills/migration "Run migration"

# Project-scoped injection
skillx run --scope project ./my-skill "Set up project"
```

## OpenAI Codex

[Codex](https://github.com/openai/codex) is OpenAI's CLI coding agent.

### Detection

skillx detects Codex by checking:

1. `codex` binary in PATH (via `which codex`)
2. `~/.codex/` directory exists

### Injection Paths

| Scope | Path |
|-------|------|
| Global | `~/.codex/skills/<skill-name>/` |
| Project | `.agents/skills/<skill-name>/` |

### Launch

skillx spawns `codex` with the prompt as a positional argument:

```bash
codex "your prompt here"
```

With YOLO mode (`--yolo`):

```bash
codex "your prompt here" --full-auto
```

### YOLO Mode

Codex's `--full-auto` flag enables fully autonomous operation without user confirmation.

```bash
skillx run --yolo --agent codex ./my-skill "Refactor the database layer"
# Equivalent to: codex "..." --full-auto
```

### Example Workflow

```bash
# Normal mode
skillx run --agent codex github:org/skills/testing "Add unit tests"

# YOLO mode
skillx run --yolo --agent codex ./my-skill "Fix all TODOs"

# With timeout and auto-confirm warnings
skillx run --yes --timeout 1h --agent codex ./my-skill "Complete refactor"
```

## Process Management

For both CLI agents, skillx handles the process lifecycle:

### Normal Exit

```
skillx spawns agent → agent completes → exit code 0 → cleanup
```

### Agent Error

```
skillx spawns agent → agent fails → exit code N → warning shown → cleanup
```

### Ctrl+C Interrupt

```
skillx spawns agent → user presses Ctrl+C → SIGKILL to agent → cleanup
```

### Timeout

```
skillx spawns agent → timeout reached → SIGKILL to agent → cleanup
```

The timeout is set with `--timeout` and supports human-friendly durations:

```bash
skillx run --timeout 5m  ./skill "prompt"   # 5 minutes
skillx run --timeout 2h  ./skill "prompt"   # 2 hours
skillx run --timeout 30s ./skill "prompt"   # 30 seconds
```

## Comparison

| Feature | Claude Code | Codex |
|---------|------------|-------|
| Binary | `claude` | `codex` |
| Lifecycle | ManagedProcess | ManagedProcess |
| Initial prompt | `--prompt` flag | Positional argument |
| YOLO flag | `--dangerously-skip-permissions` | `--full-auto` |
| Global inject | `~/.claude/skills/` | `~/.codex/skills/` |
| Project inject | `.claude/skills/` | `.agents/skills/` |

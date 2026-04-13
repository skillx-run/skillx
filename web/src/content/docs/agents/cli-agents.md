---
title: CLI Agents
description: How skillx works with Claude Code, OpenAI Codex, Gemini CLI, OpenCode, and Amp — ManagedProcess lifecycle.
---

## Overview

CLI agents run as terminal processes that skillx spawns and manages directly. skillx passes the prompt as a CLI argument, waits for the process to exit, captures the exit code, and handles interruptions gracefully.

All CLI agents use the **ManagedProcess** lifecycle mode.

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

skillx spawns `claude` with the prompt as a positional argument:

```bash
claude "your prompt here"
```

With `--print` mode (non-interactive):

```bash
claude -p "your prompt here"
```

With auto-approve mode (`--auto-approve`):

```bash
claude "your prompt here" --dangerously-skip-permissions
```

### Auto-approve Mode

Claude Code's `--dangerously-skip-permissions` flag skips all permission prompts. The agent can read, write, and execute without asking for confirmation.

```bash
skillx run --auto-approve ./examples/skills/code-review "Review all files"
# Equivalent to: claude "..." --dangerously-skip-permissions
```

### Example Workflow

```bash
# Normal mode — Claude Code will ask for permissions
skillx run ./examples/skills/code-review "Review the auth module"

# Non-interactive (print) mode — process prompt and exit
skillx run --print ./examples/skills/code-review "Review src/main.rs"

# Auto-approve mode — no permission prompts
skillx run --auto-approve ./examples/skills/code-review "Fix all lint errors"

# With timeout
skillx run --timeout 30m ./examples/skills/code-review "Review the full codebase"

# Project-scoped injection
skillx run --scope project ./examples/skills/hello-world "Set up project"
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

With `--print` mode (non-interactive):

```bash
codex exec "your prompt here"
```

With auto-approve mode (`--auto-approve`):

```bash
codex "your prompt here" --yolo
```

### Auto-approve Mode

Codex's `--yolo` flag enables fully autonomous operation without user confirmation.

```bash
skillx run --auto-approve --agent codex ./examples/skills/code-review "Refactor the database layer"
# Equivalent to: codex "..." --yolo
```

### Example Workflow

```bash
# Normal mode
skillx run --agent codex ./examples/skills/testing-guide "Add unit tests"

# Non-interactive (print) mode
skillx run --print --agent codex ./examples/skills/code-review "Review src/main.rs"

# Auto-approve mode
skillx run --auto-approve --agent codex ./examples/skills/code-review "Fix all TODOs"

# With timeout and auto-confirm warnings
skillx run --yes --timeout 1h --agent codex ./examples/skills/code-review "Complete refactor"
```

## Tier 3 CLI Agents

The following CLI agents are implemented via the data-driven `GenericAdapter`. They all use the **ManagedProcess** lifecycle with binary detection:

| Agent | Binary |
|-------|--------|
| Goose | `goose` |
| Kiro | `kiro` |
| Aider | `aider` |
| OpenClaw | `openclaw` |
| Qwen Code | `qwen-code` |
| Droid | `droid` |
| Warp | `warp` |
| OpenHands | `openhands` |
| Command Code | `command-code` |
| Mistral Vibe | `mistral-vibe` |
| Qoder | `qoder` |
| Kode | `kode` |

Each follows the same injection pattern: `~/.<name>/skills/<skill-name>/` (global) and `.<name>/skills/<skill-name>/` (project). None support auto-approve mode.

## Process Management

For all CLI agents, skillx handles the process lifecycle:

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

## Gemini CLI

[Gemini CLI](https://github.com/google-gemini/gemini-cli) is Google's command-line interface for Gemini.

### Detection

skillx detects Gemini CLI by checking:

1. `gemini` binary in PATH (via `which gemini`)
2. `~/.gemini/` directory exists

### Injection Paths

| Scope | Path |
|-------|------|
| Global | `~/.gemini/skills/<skill-name>/` |
| Project | `.gemini/skills/<skill-name>/` |

### Launch

skillx spawns `gemini` with the following arguments:

```bash
# Interactive mode with initial prompt
gemini -i "your prompt here"
```

With `--print` mode (non-interactive):

```bash
gemini -p "your prompt here"
```

### Auto-approve Mode

Gemini CLI supports auto-approve mode with the `--yolo` flag.

```bash
skillx run --auto-approve --agent gemini-cli ./examples/skills/hello-world "prompt"
# Equivalent to: gemini -i "..." --yolo
```

## OpenCode

[OpenCode](https://github.com/opencode-ai/opencode) is an open-source AI coding agent.

### Detection

skillx detects OpenCode by checking:

1. `opencode` binary in PATH
2. `~/.config/opencode/` directory exists

### Injection Paths

| Scope | Path |
|-------|------|
| Global | `~/.opencode/skills/<skill-name>/` |
| Project | `.opencode/skills/<skill-name>/` |

### Launch

skillx spawns `opencode` with the prompt as a positional argument:

```bash
opencode "your prompt here"
```

With `--print` mode (non-interactive, auto-approves all permissions):

```bash
opencode run "your prompt here"
```

### Auto-approve Mode

Not supported.

## Amp

[Amp](https://github.com/nicholasgasior/amp) is an AI-powered coding agent.

### Detection

skillx detects Amp by checking:

1. `amp` binary in PATH
2. `~/.amp/` directory exists

### Injection Paths

| Scope | Path |
|-------|------|
| Global | `~/.config/agents/skills/<skill-name>/` |
| Project | `.agents/skills/<skill-name>/` |

### Launch

skillx spawns `amp` with the `-x` (execute) flag for prompt delivery:

```bash
amp -x "your prompt here"
```

### Auto-approve Mode

Amp supports auto-approve mode with the `--dangerously-allow-all` flag.

```bash
skillx run --auto-approve --agent amp ./examples/skills/hello-world "prompt"
# Equivalent to: amp -x "..." --dangerously-allow-all
```

## Comparison

| Feature | Claude Code | Codex | Gemini CLI | OpenCode | Amp |
|---------|------------|-------|------------|----------|-----|
| Binary | `claude` | `codex` | `gemini` | `opencode` | `amp` |
| Lifecycle | ManagedProcess | ManagedProcess | ManagedProcess | ManagedProcess | ManagedProcess |
| Initial prompt | Positional arg | Positional arg | `-i` flag | Positional arg | `-x` flag |
| Print mode | `-p` flag | `exec` subcommand | `-p` flag | `run` subcommand | N/A |
| Auto-approve flag | `--dangerously-skip-permissions` | `--yolo` | `--yolo` | N/A | `--dangerously-allow-all` |
| Global inject | `~/.claude/skills/` | `~/.codex/skills/` | `~/.gemini/skills/` | `~/.opencode/skills/` | `~/.config/agents/skills/` |
| Project inject | `.claude/skills/` | `.agents/skills/` | `.gemini/skills/` | `.opencode/skills/` | `.agents/skills/` |

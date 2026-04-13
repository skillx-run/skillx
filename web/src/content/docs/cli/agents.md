---
title: "skillx agents"
description: Reference for the skillx agents command — list detected agent environments.
---

## Synopsis

```bash
skillx agents [options]
```

Detect and display all agent environments available on your system. This is useful for verifying which agents skillx can use and how they are configured.

## Options

| Flag | Default | Description |
|------|---------|-------------|
| `--all` | — | Show all known agents, including those not detected |

## Output

By default, only detected agents are shown:

```bash
skillx agents
```

```
Agent Environments

  Claude Code [✓ detected]
    claude binary found
    Lifecycle: ManagedProcess
    Auto-approve: --dangerously-skip-permissions

  Cursor [✓ detected]
    Cursor process detected
    Lifecycle: FileInjectAndWait

  Universal Agent [✓ detected]
    universal fallback (.agents/skills/)
    Lifecycle: FileInjectAndWait
```

### With --all

```bash
skillx agents --all
```

```
Agent Environments

  Claude Code [✓ detected]
    claude binary found
    Lifecycle: ManagedProcess
    Auto-approve: --dangerously-skip-permissions

  OpenAI Codex [✗ not found]

  GitHub Copilot [✗ not found]

  Cursor [✓ detected]
    Cursor process detected
    Lifecycle: FileInjectAndWait

  Universal Agent [✓ detected]
    universal fallback (.agents/skills/)
    Lifecycle: FileInjectAndWait
```

## Agent Details

For each detected agent, the output shows:

| Field | Description |
|-------|-------------|
| **Status** | Whether the agent was detected on the system |
| **Info** | How the agent was detected (binary found, process running, etc.) |
| **Lifecycle** | `ManagedProcess` (skillx spawns it) or `FileInjectAndWait` (inject files, wait for user) |
| **Auto-approve** | The flags passed in `--auto-approve` mode, if supported |

## Detection Methods

Each agent is detected differently:

| Agent | Detection |
|-------|-----------|
| Claude Code | `claude` binary in PATH or `~/.claude/` directory exists |
| OpenAI Codex | `codex` binary in PATH or `~/.codex/` directory exists |
| GitHub Copilot | GitHub Copilot extension in `~/.vscode/extensions/` |
| Cursor | `cursor` binary in PATH or Cursor process running |
| Universal | Always available (fallback adapter) |

## Selection Priority

When `skillx run` auto-detects agents:

1. If exactly one agent is detected (excluding Universal), it is used automatically
2. If multiple agents are detected, an interactive prompt lets you choose
3. If no agents are detected, the Universal adapter is used as a fallback
4. The `--agent` flag overrides auto-detection entirely

You can also set a preferred agent in `~/.skillx/config.toml`:

```toml
[agent.defaults]
preferred = "claude-code"
```

## Built-in Agents

| Name | Display Name | Lifecycle | Auto-approve Args |
|------|-------------|-----------|-----------|
| `claude-code` | Claude Code | ManagedProcess | `--dangerously-skip-permissions` |
| `codex` | OpenAI Codex | ManagedProcess | `--full-auto` |
| `copilot` | GitHub Copilot | FileInjectAndWait | — |
| `cursor` | Cursor | FileInjectAndWait | — |
| `universal` | Universal Agent | FileInjectAndWait | — |

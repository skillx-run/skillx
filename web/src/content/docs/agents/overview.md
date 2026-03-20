---
title: Agent System Overview
description: How skillx detects, selects, and adapts to different AI coding agents.
---

## What Are Agents?

In skillx, an "agent" is an AI coding assistant that can read instructions from files and act on them. skillx supports 11 agents across two categories: CLI processes (Claude Code, Codex, Gemini CLI, OpenCode, Amp) and IDE integrations (Copilot, Cursor, Windsurf, Cline, Roo Code), plus a Universal fallback.

Each agent has different conventions for where skill files should be placed, how to launch, and what flags are available. skillx abstracts these differences behind a unified interface.

## Agent Detection

When you run `skillx run` without specifying `--agent`, skillx auto-detects which agents are available:

```bash
skillx agents
```

Detection methods vary by agent:

| Agent | How Detected |
|-------|-------------|
| Claude Code | `claude` binary in PATH or `~/.claude/` directory |
| OpenAI Codex | `codex` binary in PATH or `~/.codex/` directory |
| Gemini CLI | `gemini` binary in PATH or `~/.gemini/` directory |
| OpenCode | `opencode` binary in PATH or `~/.config/opencode/` directory |
| Amp | `amp` binary in PATH or `~/.amp/` directory |
| GitHub Copilot | Copilot extension in `~/.vscode/extensions/` |
| Cursor | `cursor` binary in PATH or Cursor process running |
| Windsurf | `windsurf` binary in PATH or Windsurf process running |
| Cline | VS Code extension `saoudrizwan.claude-dev` |
| Roo Code | VS Code extension `rooveterinaryinc.roo-cline` |
| Universal | Always available (fallback) |

## Selection Logic

The selection process follows this flow:

```
1. --agent flag provided?
   YES → use that agent (error if not found in registry)
   NO  → continue to detection

2. How many agents detected?
   0 → use Universal fallback
   1 → use that agent automatically
   2+ → show interactive selector

3. config.toml has preferred agent?
   YES → use preferred if it's among detected agents
   NO  → show interactive selector
```

### Explicit Selection

```bash
skillx run --agent claude-code ./my-skill "prompt"
skillx run --agent codex ./my-skill "prompt"
skillx run --agent cursor ./my-skill "prompt"
skillx run --agent universal ./my-skill "prompt"
```

### Preferred Agent

Set a default in `~/.skillx/config.toml`:

```toml
[agent.defaults]
preferred = "claude-code"
```

## Lifecycle Modes

Agents operate in one of two lifecycle modes:

### ManagedProcess

skillx spawns the agent as a child process, passes the prompt, and waits for it to exit.

```
skillx → spawn agent process → wait → cleanup
```

**Agents**: Claude Code, OpenAI Codex, Gemini CLI, OpenCode, Amp

Features:
- Prompt passed as CLI argument
- Exit code captured
- Ctrl+C kills the agent process
- `--timeout` support
- `--yolo` mode for permission-skipping (Claude Code, Codex, Gemini CLI)

### FileInjectAndWait

skillx injects files into the agent's directory, optionally copies the prompt to the clipboard, and waits for the user to press Enter.

```
skillx → inject files → (clipboard) → wait for Enter → cleanup
```

**Agents**: GitHub Copilot, Cursor, Windsurf, Cline, Roo Code, Universal

Features:
- Prompt copied to system clipboard
- User signals completion by pressing Enter
- Ctrl+C triggers cleanup
- `--timeout` support

## Injection Paths

Each agent has specific directories where it looks for skill files:

| Agent | Global Scope | Project Scope |
|-------|-------------|---------------|
| Claude Code | `~/.claude/skills/<name>/` | `.claude/skills/<name>/` |
| Codex | `~/.codex/skills/<name>/` | `.agents/skills/<name>/` |
| Gemini CLI | `~/.gemini/skills/<name>/` | `.gemini/skills/<name>/` |
| OpenCode | `~/.opencode/skills/<name>/` | `.opencode/skills/<name>/` |
| Amp | `~/.amp/skills/<name>/` | `.amp/skills/<name>/` |
| Copilot | `~/.github/skills/<name>/` | `.github/skills/<name>/` |
| Cursor | `~/.cursor/skills/<name>/` | `.cursor/skills/<name>/` |
| Windsurf | `~/.windsurf/skills/<name>/` | `.windsurf/skills/<name>/` |
| Cline | `~/.cline/skills/<name>/` | `.cline/skills/<name>/` |
| Roo Code | `~/.roo/skills/<name>/` | `.roo/skills/<name>/` |
| Universal | `~/.agents/skills/<name>/` | `.agents/skills/<name>/` |

The scope is controlled by `--scope`:

```bash
skillx run --scope global ./my-skill "prompt"   # default
skillx run --scope project ./my-skill "prompt"   # project-local
```

## YOLO Mode

CLI agents can skip their built-in permission prompts:

| Agent | YOLO Flag |
|-------|-----------|
| Claude Code | `--dangerously-skip-permissions` |
| OpenAI Codex | `--full-auto` |
| Gemini CLI | `--sandbox=none` |
| OpenCode | Not supported |
| Amp | Not supported |
| Copilot | Not supported |
| Cursor | Not supported |
| Windsurf | Not supported |
| Cline | Not supported |
| Roo Code | Not supported |
| Universal | Not supported |

```bash
skillx run --yolo ./my-skill "prompt"
```

:::caution
YOLO mode gives the agent unrestricted access to your system. Only use with trusted skills.
:::

## Next Steps

- [CLI Agents](/agents/cli-agents/) — Claude Code and Codex details
- [IDE Agents](/agents/ide-agents/) — Copilot and Cursor details
- [Universal](/agents/universal/) — the fallback adapter
- [Agent Adapters Guide](/guides/agent-adapters/) — write your own adapter

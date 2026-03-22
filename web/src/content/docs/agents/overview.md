---
title: Agent System Overview
description: How skillx detects, selects, and adapts to different AI coding agents.
---

## What Are Agents?

In skillx, an "agent" is an AI coding assistant that can read instructions from files and act on them. skillx supports **32 built-in agents** across three tiers, plus custom agents via `config.toml`. Each agent has different conventions for where skill files should be placed, how to launch, and what flags are available. skillx abstracts these differences behind a unified interface.

## Agent Tiers

### Tier 1 — Dedicated Adapters (CLI)

| Agent | Type | Detection | YOLO |
|-------|------|-----------|------|
| Claude Code | CLI | `claude` binary or `~/.claude/` | `--dangerously-skip-permissions` |
| OpenAI Codex | CLI | `codex` binary or `~/.codex/` | `--full-auto` |

### Tier 1 — Dedicated Adapters (IDE)

| Agent | Type | Detection | YOLO |
|-------|------|-----------|------|
| GitHub Copilot | IDE | VS Code extension `github.copilot-*` | — |
| Cursor | IDE | `cursor` binary or process | — |

### Tier 2 — Dedicated Adapters

| Agent | Type | Detection | YOLO |
|-------|------|-----------|------|
| Gemini CLI | CLI | `gemini` binary or `~/.gemini/` | `--sandbox=none` |
| OpenCode | CLI | `opencode` binary or `~/.config/opencode/` | — |
| Amp | CLI | `amp` binary or `~/.amp/` | — |
| Windsurf | IDE | `windsurf` binary or process | — |
| Cline | IDE | VS Code extension `saoudrizwan.claude-dev-*` | — |
| Roo Code | IDE | VS Code extension `rooveterinaryinc.roo-cline-*` | — |

### Tier 3 — Generic Adapters (21 agents)

These agents are implemented via the data-driven `GenericAdapter` and require no custom code:

**CLI agents** (ManagedProcess, binary detection):
Goose, Kiro, Aider, OpenClaw, Qwen Code, Droid, Warp, OpenHands, Command Code, Mistral Vibe, Qoder, Kode

**IDE agents** (FileInjectAndWait):
- VS Code extension detection: Kilo Code, Augment, Continue, CodeBuddy, Antigravity, Zencoder, Junie
- Process detection: Trae
- Config directory: Replit Agent

### Custom Agents

Define your own agents in `~/.skillx/config.toml`:

```toml
[[custom_agents]]
name = "my-agent"
display_name = "My Agent"
binary = "my-agent"
config_dir = ".my-agent"
lifecycle = "managed_process"   # or "file_inject_and_wait"
supports_prompt = true
supports_yolo = false
```

Custom agents use the same `GenericAdapter` as Tier 3 agents.

### Universal Fallback

The `universal` adapter is always available as a last resort. It injects files into `~/.agents/skills/` (global) or `.agents/skills/` (project).

## Agent Detection

When you run `skillx run` without specifying `--agent`, skillx auto-detects which agents are available:

```bash
skillx agents          # Show detected agents
skillx agents --all    # Show all 32 known agents
```

## Selection Logic

The selection process follows this flow:

```
1. --agent flag provided?
   YES → use that agent (error if not found in registry)
   NO  → continue to detection

2. skillx.toml has [agent].preferred?
   YES → use preferred if it's among detected agents
   NO  → continue

3. config.toml has preferred agent?
   YES → use preferred if it's among detected agents
   NO  → continue

4. How many agents detected?
   0 → use Universal fallback
   1 → use that agent automatically
   2+ → show interactive selector
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

Or per-project in `skillx.toml`:

```toml
[agent]
preferred = "claude-code"
```

## Lifecycle Modes

Agents operate in one of two lifecycle modes:

### ManagedProcess

skillx spawns the agent as a child process, passes the prompt, and waits for it to exit.

```
skillx → spawn agent process → wait → cleanup
```

**Agents**: Claude Code, Codex, Gemini CLI, OpenCode, Amp, and 12 Tier 3 CLI agents

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

**Agents**: Copilot, Cursor, Windsurf, Cline, Roo Code, Universal, and 9 Tier 3 IDE agents

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

Tier 3 and custom agents follow the pattern `~/.<config_dir>/skills/<name>/` (global) and `.<config_dir>/skills/<name>/` (project).

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
| All others | Not supported |

```bash
skillx run --yolo ./my-skill "prompt"
```

:::caution
YOLO mode gives the agent unrestricted access to your system. Only use with trusted skills.
:::

## Next Steps

- [CLI Agents](/agents/cli-agents/) — CLI agent details
- [IDE Agents](/agents/ide-agents/) — IDE agent details
- [Universal](/agents/universal/) — the fallback adapter
- [Agent Adapters Guide](/guides/agent-adapters/) — write your own adapter

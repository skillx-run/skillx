---
title: Universal Adapter
description: The fallback agent adapter that works with any AI agent following the .agents/skills/ convention.
---

## Overview

The Universal adapter is a fallback that works with any AI agent that can read files from a known directory. It uses the `.agents/skills/` convention and is always available — it cannot be "not detected."

## When Is It Used?

The Universal adapter is selected when:

1. No other agent is detected on the system
2. You explicitly request it: `--agent universal`
3. The agent you want to use isn't in the built-in registry

## Injection Paths

| Scope | Path |
|-------|------|
| Global | `~/.agents/skills/<skill-name>/` |
| Project | `.agents/skills/<skill-name>/` |

The `.agents/skills/` directory convention is designed to be agent-agnostic. Any agent that supports reading from this path will work.

## Lifecycle

The Universal adapter uses the **FileInjectAndWait** lifecycle:

1. Copy skill files to the injection path
2. Copy prompt to clipboard (if available)
3. Print: `"Skill injected to .agents/skills/. Use your preferred agent."`
4. Wait for Enter or Ctrl+C
5. Clean up injected files

## Usage

```bash
# Explicit selection
skillx run --agent universal ./my-skill "Analyze this code"

# Automatic fallback (when no agents detected)
skillx run ./my-skill "Analyze this code"
```

## Configuration

The Universal adapter does not support:

- **YOLO mode** — it doesn't know which agent you'll use
- **Initial prompt delivery** — prompt is clipboard-only
- **Process management** — no process to spawn

## Use Cases

### Agent Not in Registry

If you use an agent that skillx doesn't have a built-in adapter for, the Universal adapter lets you still use the fetch-scan-inject-clean lifecycle:

```bash
skillx run --agent universal github:org/skills/analysis "prompt"
# Files injected to ~/.agents/skills/analysis/
# Now open your agent and point it to the skill
```

### Multi-Agent Workflows

Use Universal when you want to inject a skill and then manually choose which agent to use:

```bash
skillx run --scope project --agent universal ./my-skill "prompt"
# Files at .agents/skills/my-skill/ — use with any agent
```

### Headless Environments

In environments where no specific agent is installed (CI, remote servers), Universal provides a way to scan and inject skills:

```bash
# Inject skill files for later use
skillx run --agent universal ./deployment-skill "Deploy to staging"
```

## Extending with Custom Agents

If you frequently use an agent that isn't in the built-in registry, consider writing a custom adapter. See [Agent Adapters Guide](/guides/agent-adapters/) for instructions.

The Universal adapter exists so that skillx is useful even without a dedicated adapter for your agent of choice.

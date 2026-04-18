---
title: "Hello World"
description: Step-by-step tutorial for the hello-world example skill — scanning, running, and understanding the output.
---

The `hello-world` skill is the simplest possible skillx skill. It demonstrates the full lifecycle: scan, run, observe, cleanup.

## SKILL.md

Here is the complete content of `examples/skills/hello-world/SKILL.md`:

```markdown
---
name: hello-world
description: A simple greeting skill that demonstrates skillx capabilities
author: skillx-run
version: "1.0.0"
license: MIT
tags:
  - example
  - starter
---

# Hello World Skill

You are an assistant using the skillx Hello World skill.

## Instructions

When invoked, greet the user warmly and explain what skillx skills are:

1. **skillx** lets you run any agent skill without installing it
2. **Skills** are reusable instruction sets that enhance your AI coding agent
3. This skill was injected via `skillx run` — it will be automatically cleaned up when the session ends

Then help the user with whatever task they describe.

## Example Interactions

- "hello" → Greet and briefly introduce skillx
- "help me write a function" → Greet, then assist with coding
- "what can you do?" → Explain your capabilities enhanced by this skill
```

### Structure

This is a single-file skill — just a `SKILL.md` with no scripts or references:

```
hello-world/
└── SKILL.md
```

The frontmatter provides metadata (name, description, author, version, license, tags). The body contains the instructions the agent will follow.

## Step 1: Scan

Before running any skill, scan it to verify it is safe:

```bash
skillx scan ./examples/skills/hello-world
```

Expected output:

```
  Scanning  hello-world
  ────────────────────────────────
  ✓ PASS — No issues found

  Files scanned: 1
  Risk level: PASS
```

The skill passes cleanly because it contains no dangerous patterns — no external URLs, no shell commands, no sensitive directory references.

## Step 2: Run

Run the skill with a prompt:

```bash
skillx run ./examples/skills/hello-world "Hello, what can you do?"
```

skillx performs the full lifecycle:

1. **Resolve** — Finds `SKILL.md` in the local path
2. **Scan** — Checks for security issues (PASS)
3. **Detect agent** — Identifies your active AI coding agent
4. **Inject** — Copies the skill into the agent's instruction directory
5. **Wait** — Keeps the session active while you interact with the agent
6. **Cleanup** — Removes all injected files when you press `Ctrl+C`

## Step 3: Observe

Once injected, your agent has access to the skill's instructions. The agent will greet you and explain what skillx is, then help with your request.

When you are done, press `Ctrl+C` to end the session. skillx removes the injected files and archives the session manifest.

## Running with Different Agents

By default, skillx auto-detects your active agent. You can override this with `--agent`:

### Claude Code

```bash
skillx run ./examples/skills/hello-world --agent claude-code "Hello"
```

Injects into Claude Code's `CLAUDE.md` instruction file.

### Codex

```bash
skillx run ./examples/skills/hello-world --agent codex "Hello"
```

Injects into Codex's instruction directory.

### Cursor

```bash
skillx run ./examples/skills/hello-world --agent cursor "Hello"
```

Injects into Cursor's `.cursor/rules/` directory.

### Universal (Fallback)

```bash
skillx run ./examples/skills/hello-world --agent universal "Hello"
```

Copies the skill to the project root and displays the content for manual use with any agent.

## Running from GitHub

You do not need a local clone to try this skill:

```bash
skillx run github:skillx-run/skillx/examples/skills/hello-world "Hello"
```

skillx fetches the skill from GitHub, caches it locally, scans it, and injects it — all in one command.

## Why this example exists

This example is the smallest complete skill in the repository, so it is the fastest way to understand what a runnable `SKILL.md` looks like.

## Next Steps

Use [Famous Skills](/getting-started/famous-skills/), return to the [Official Examples](/examples/overview/), or continue with [Writing Skills](/guides/writing-skills/) when you are ready to build your own skill.

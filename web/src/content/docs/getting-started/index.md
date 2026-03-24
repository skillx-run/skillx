---
title: Introduction to skillx
description: What skillx is, why it exists, and how its fetch-scan-inject-run-clean lifecycle works.
---

## What is skillx?

**skillx** is a cross-platform CLI tool that provides a zero-install experience for Agent Skills — reusable prompt-and-script packages that extend AI coding agents. Think of it as **npx for Agent Skills**: one command handles the complete lifecycle so you never manually clone, copy, or clean up skill files.

```bash
skillx run github:skillx-run/skillx.run/examples/skills/name-poem "Your Name"
```

That single command fetches the skill, scans it for security issues, injects it into your agent's context, launches the agent, waits for completion, and cleans everything up.

## Why skillx?

AI coding agents (Claude Code, Codex, Copilot, Cursor) can follow instructions from markdown files, but sharing and running those instructions is a manual process:

1. Clone a repo containing the skill
2. Copy the right files into the right agent-specific directory
3. Hope the skill doesn't contain anything malicious
4. Remember to clean up when you're done

Every other skill manager requires permanent installation into your project. skillx doesn't. One command fetches, scans, runs, and auto-cleans — no files are permanently added to your project. When you do want persistence, `skillx install` is there, but it's opt-in, not the default.

## The Lifecycle

Every `skillx run` goes through five phases:

### 1. Fetch

Resolve the source (local path, `github:` shorthand, or full GitHub URL), download if remote, and cache for future use.

```
local path    →  use directly
github:org/repo  →  download tarball via GitHub API
https://github.com/...  →  parse URL, download tarball
```

### 2. Scan

Run the built-in security scanner against the skill's files. The scanner checks SKILL.md for prompt injection, scripts for dangerous operations, and resources for disguised binaries. Every finding gets a risk level: PASS, INFO, WARN, DANGER, or BLOCK.

### 3. Inject

Copy the skill's files into the agent's expected directory (e.g., `~/.claude/skills/` for Claude Code). Each file is hashed with SHA-256 and recorded in a session manifest so cleanup is reliable.

### 4. Run

Launch the agent with the skill in context. CLI agents (Claude Code, Codex) are spawned as child processes. IDE agents (Copilot, Cursor) get file injection plus clipboard copy of the prompt, then skillx waits for you to press Enter.

### 5. Clean

Remove all injected files, archive the session manifest to `~/.skillx/history/`, and recover any orphaned sessions from previous interrupted runs.

## Core Concepts

### Skills

A skill is a directory containing at minimum a `SKILL.md` file — a markdown document with YAML frontmatter and instructions for the agent. Skills can also include:

- `scripts/` — helper scripts the agent can execute
- `references/` — supporting documents, examples, templates

### Agents

skillx detects which AI agents are installed on your system and adapts its behavior accordingly. It supports 32 built-in agents including Claude Code, Codex, Copilot, Cursor, Gemini CLI, and more, plus custom agents via config.

### Security Scanner

Every skill is scanned before injection. The scanner uses 23 rules across three categories (Markdown, Script, Resource) to detect prompt injection, credential access, destructive operations, and more.

### Sessions

Each run creates a session with a unique ID. The session tracks injected files, their checksums, the scan report, and timing information. Sessions are stored in `~/.skillx/active/` during execution and archived to `~/.skillx/history/` afterward.

## Quick Example

```bash
# Install skillx
curl -fsSL https://skillx.run/install.sh | sh

# Run a skill from GitHub
skillx run github:skillx-run/skillx.run/examples/skills/code-review "Review my auth module"

# Scan a skill without running it
skillx scan ./my-local-skill

# See which agents are installed
skillx agents
```

## Next Steps

- [Installation](/getting-started/installation/) — get skillx on your machine
- [First Run](/getting-started/first-run/) — walk through a complete example
- [Writing Skills](/guides/writing-skills/) — create your own skills

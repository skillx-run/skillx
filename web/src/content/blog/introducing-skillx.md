---
title: "Introducing skillx v0.1"
description: "npx for Agent Skills — fetch, scan, inject, run, clean in one command"
date: 2026-03-20
author: "skillx team"
---

> **Note:** This post describes skillx v0.1. For the latest features (32 agents, 10 source types, 23 scanner rules), see the [documentation](/docs).

# Introducing skillx v0.1

We're excited to announce **skillx** — a cross-platform CLI tool that brings the `npx` experience to Agent Skills.

## The Problem

Agent Skills are spreading across repositories, registries, and platforms. Running a skill today requires:

1. Finding it (GitHub? skills.sh? A colleague's repo?)
2. Downloading it manually
3. Checking if it's safe (does anyone actually do this?)
4. Copying files to the right agent directory
5. Running your agent
6. Cleaning up afterward

That's six steps before you even start working.

## The Solution

```bash
skillx run github:anthropics/skills/pdf-processing "Extract tables from report.pdf"
```

One command. skillx handles the entire lifecycle:

- **Fetch** — Pulls the skill from GitHub (or any supported source)
- **Scan** — Runs a security scan with 20+ rules
- **Gate** — Shows you findings and asks for confirmation
- **Inject** — Places files in the right agent directory
- **Run** — Launches your agent with the prompt
- **Clean** — Removes everything when done

## What's in v0.1

### Supported Agents

- **Claude Code** — Full lifecycle with `--yolo` mode
- **OpenAI Codex** — Full lifecycle with `--full-auto`
- **GitHub Copilot** — File injection + clipboard
- **Cursor** — File injection + clipboard
- **Universal** — Fallback for any agent

### Security Scanner

20 rules across three categories:

- **MD-001~006** — SKILL.md analysis (prompt injection, sensitive dirs, external URLs)
- **SC-001~011** — Script analysis (binary detection, eval/exec, rm -rf, privilege escalation)
- **RS-001~003** — Resource analysis (disguised files, large files, executables in references)

Five risk levels: **PASS → INFO → WARN → DANGER → BLOCK**

### Source Types

- Local paths (`./my-skill/`)
- GitHub prefix (`github:owner/repo/path[@ref]`)
- GitHub URLs (`https://github.com/...`)

## What's Next

- **v0.2** — `install`/`uninstall`/`list`/`update`, more platforms (GitLab, Bitbucket), Tier 2 agents
- **v0.3** — 20+ agent adapters, custom URL patterns
- **v0.4** — `registry.skillx.run` for skill discovery

## Get Started

```bash
cargo install skillx
skillx run ./my-skill "Do the thing"
```

Read the [documentation](/docs) or check out the [GitHub repository](https://github.com/skillx-run/skillx).

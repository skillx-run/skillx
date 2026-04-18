---
title: "Commit Message"
description: Generate conventional commit messages from staged changes — designed for piped input and non-interactive workflows.
---

The `commit-message` skill generates clear, conventional commit messages from diffs. It is designed for piped input (`--stdin`) and non-interactive workflows, making it ideal for shell aliases and Git hooks.

## SKILL.md Overview

The skill defines:

- **Conventional Commits format** — `<type>(<scope>): <subject>` with an optional body
- **Type vocabulary** — feat, fix, refactor, docs, test, chore, perf, style
- **Rules** — 72-char subject lines, imperative mood, explain "why" not "what"
- **Split guidance** — If multiple logical changes are detected, suggest separate commits

```
commit-message/
└── SKILL.md
```

## Basic Usage

### Run from GitHub

Run the official example directly from the docs:

```bash
skillx run github:skillx-run/skillx/examples/skills/commit-message "Generate a commit message for my staged changes"
```

### Run from a Local Clone of this Repository

If you are working inside a clone of `skillx-run/skillx`, use the repository path:

```bash
skillx run ./examples/skills/commit-message "Generate a commit message for my staged changes"
```

### Interactive Mode

Run the skill and describe what you changed:

```bash
skillx run github:skillx-run/skillx/examples/skills/commit-message "Generate a commit message for my staged changes"
```

The agent reads your staged diff and produces a commit message.

### Piped Input with --stdin

The most powerful usage pipes a diff directly into the skill:

```bash
git diff --staged | skillx run github:skillx-run/skillx/examples/skills/commit-message --stdin
```

This sends the staged diff as input to the agent. The skill's instructions tell the agent to analyze the diff and generate a conventional commit message.

### Print Mode (Non-Interactive)

Use `--print` to output the skill content without launching an interactive session. This is useful for scripting:

```bash
git diff --staged | skillx run github:skillx-run/skillx/examples/skills/commit-message --stdin --print
```

In `--print` mode, skillx outputs the skill instructions to stdout and exits immediately — no agent session is created, no cleanup is needed.

## Workflow Examples

### Quick Commit Message

A one-liner to generate and review a commit message:

```bash
git diff --staged | skillx run github:skillx-run/skillx/examples/skills/commit-message --stdin
```

The agent outputs something like:

```
feat(scanner): add SARIF 2.1.0 output format

Add structured SARIF output to the scan report formatter, enabling
integration with GitHub Code Scanning and other SARIF-compatible
tools.
```

### Review Before Committing

Generate the message, review it, then commit:

```bash
# Stage your changes
git add -p

# Generate the commit message
skillx run github:skillx-run/skillx/examples/skills/commit-message "Generate a commit message for my staged changes"

# Copy the suggested message and commit
git commit -m "feat(scanner): add SARIF 2.1.0 output format"
```

### Diff of Recent Commits

Generate a message for squashing recent commits:

```bash
git diff HEAD~3 | skillx run github:skillx-run/skillx/examples/skills/commit-message --stdin
```

### PR Description

The skill also works for pull request descriptions by providing a broader diff:

```bash
git diff main...HEAD | skillx run github:skillx-run/skillx/examples/skills/commit-message --stdin
```

## The Conventional Commits Format

The skill enforces the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
<type>(<scope>): <subject>

<body>
```

### Types

| Type | When to Use |
|------|-------------|
| `feat` | New feature |
| `fix` | Bug fix |
| `refactor` | Code change that neither fixes a bug nor adds a feature |
| `docs` | Documentation changes |
| `test` | Adding or updating tests |
| `chore` | Build process, CI, dependencies |
| `perf` | Performance improvement |
| `style` | Formatting, whitespace (no logic change) |

### Rules

- **Subject line** — Max 72 characters, imperative mood ("add" not "added"), no trailing period
- **Body** — Explains the "why" not the "what" (the diff shows the what)
- **Multiple changes** — If the diff contains multiple logical changes, the skill suggests splitting into separate commits

## Team Configuration

Add the commit-message skill to your project's `skillx.toml` for consistent commit messages across the team:

```toml
[skills]
commit-message = "github:skillx-run/skillx/examples/skills/commit-message"
```

Then any team member can run:

```bash
git diff --staged | skillx run --stdin
```

The `skillx run` command (with no source argument) reads from `skillx.toml` and uses the configured skills.

## Why this example exists

This example exists to show how a skill can read staged changes, infer intent, and emit a commit message that fits non-interactive shell workflows and Git habits.

## Next Steps

If you want a companion pattern for review workflows, open [Code Review](/examples/code-review/). If you want to build your own command-line skill around a different Git task, use [Writing Skills](/guides/writing-skills/). If you are comparing this with upstream workflows, check [Famous Skills](/getting-started/famous-skills/).

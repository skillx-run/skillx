---
title: Example Skills
description: Overview of official example skills included in the skillx repository — starters, practical tools, and security demonstrations.
---

The skillx repository includes six example skills in the `examples/skills/` directory. These serve as both learning resources and templates for writing your own skills.

## Categories

### Showcase

| Skill | Description |
|-------|-------------|
| [name-poem](/examples/name-poem) | Generate beautiful poems from names — classical Chinese acrostic poetry, haiku, sijo, and more |

### Starter

| Skill | Description |
|-------|-------------|
| [hello-world](/examples/hello-world) | A minimal greeting skill — the simplest possible SKILL.md |

### Practical

| Skill | Description |
|-------|-------------|
| [code-review](/examples/code-review) | Structured code review with severity levels and actionable feedback |
| [testing-guide](/examples/testing-guide) | Test writing guidance with patterns for unit, integration, and edge case coverage |
| [commit-message](/examples/commit-message) | Generate conventional commit messages from staged changes |

### Security Demo

| Skill | Description |
|-------|-------------|
| dangerous-example | Intentionally malicious skill for demonstrating the scanner — **blocked by default** |

## Running Examples

### From a Local Clone

If you have the skillx repository cloned locally, run examples directly from the filesystem:

```bash
# Run the name-poem skill
skillx run ./examples/skills/name-poem "Your Name"

# Run the hello-world skill
skillx run ./examples/skills/hello-world "Hello"

# Scan before running
skillx scan ./examples/skills/code-review

# Run with a specific agent
skillx run ./examples/skills/commit-message --agent claude-code "Generate a commit message"
```

### From GitHub

You can run the examples without cloning the repository. skillx fetches them directly from GitHub:

```bash
# Run name-poem from GitHub
skillx run github:skillx-run/skillx.run/examples/skills/name-poem "Your Name"

# Run hello-world from GitHub
skillx run github:skillx-run/skillx.run/examples/skills/hello-world "Hello"

# Run code-review from GitHub
skillx run github:skillx-run/skillx.run/examples/skills/code-review "Review my last commit"

# Run testing-guide from GitHub
skillx run github:skillx-run/skillx.run/examples/skills/testing-guide "Help me write tests for utils.ts"
```

### Scanning the Dangerous Example

The `dangerous-example` skill is designed to be caught by the scanner. Try scanning it to see what skillx detects:

```bash
skillx scan ./examples/skills/dangerous-example
```

The scanner will flag multiple rules — prompt injection (MD-001), sensitive directory access (MD-002), external URLs (MD-003), dangerous shell commands (SC-003, SC-004, SC-006), and `eval` usage (SC-002). If you attempt to `skillx run` this skill, the gate will **block** execution.

## Using Examples as Templates

Each example demonstrates a different skill pattern:

- **name-poem** — Multilingual prompt design with cultural awareness and structured output
- **hello-world** — Minimal SKILL.md with frontmatter and simple instructions
- **code-review** — Structured output format with severity levels
- **testing-guide** — Multi-file skill with a `references/` directory
- **commit-message** — Designed for piped input (`--stdin` mode)
- **dangerous-example** — Shows what the scanner catches (do not use as a template)

To create your own skill based on an example:

```bash
# Copy the template
cp -r examples/skills/hello-world my-skill

# Edit SKILL.md
$EDITOR my-skill/SKILL.md

# Test it
skillx scan my-skill
skillx run ./my-skill "Test prompt"
```

See the [Writing Skills](/guides/writing-skills) guide for a complete walkthrough.

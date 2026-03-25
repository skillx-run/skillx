---
title: "Code Review"
description: A structured code review skill with severity levels, actionable feedback, and team configuration examples.
---

The `code-review` skill turns your AI agent into a structured code reviewer. It defines a review process, output format, severity levels, and review guidelines — all in a single SKILL.md.

## Design Rationale

Code reviews are one of the most common uses for AI coding agents, but unstructured reviews produce inconsistent results. This skill solves that by defining:

- **A repeatable process** — five steps from understanding context to checking performance
- **Severity levels** — CRITICAL, WARNING, SUGGESTION, NOTE — so the author knows what to fix first
- **A structured format** — `[SEVERITY] file:line — description` for every finding
- **A summary template** — approve, request changes, or flag for discussion

This ensures every review follows the same pattern regardless of which team member runs it.

## SKILL.md

The skill is a single-file directory:

```
code-review/
└── SKILL.md
```

Key sections of the SKILL.md:

- **Review Process** — 5-step checklist: context, correctness, style, security, performance
- **Output Format** — `[SEVERITY] file:line — Brief description` with a suggested fix
- **Severity Levels** — CRITICAL (bugs, security), WARNING (performance, code smells), SUGGESTION (style), NOTE (observations)
- **Guidelines** — Be specific, constructive, proportional; acknowledge good code
- **Summary** — Counts by severity, overall verdict, key concern

## Usage Scenarios

### Review the Last Commit

```bash
skillx run ./examples/skills/code-review "Review the changes in the last commit"
```

The agent reads the recent diff and produces a structured review.

### Review Staged Changes

```bash
skillx run ./examples/skills/code-review "Review my staged changes before I commit"
```

Useful as a pre-commit quality check.

### Review a Specific File

```bash
skillx run ./examples/skills/code-review "Review src/handlers/auth.rs for security issues"
```

Focus the review on a specific file or concern.

### Pipe a Diff Directly

For non-interactive use, pipe the diff via stdin:

```bash
git diff HEAD~3 | skillx run ./examples/skills/code-review --stdin
```

This sends the diff of the last 3 commits as input. The agent receives the diff content and reviews it using the structured format defined in the skill.

You can also pipe a PR diff:

```bash
gh pr diff 42 | skillx run ./examples/skills/code-review --stdin
```

## Scan Output

Scanning the code-review skill:

```bash
skillx scan ./examples/skills/code-review
```

```
  Scanning  code-review
  ────────────────────────────────
  ✓ PASS — No issues found

  Files scanned: 1
  Risk level: PASS
```

The skill is purely instructional — no scripts, no URLs, no sensitive references — so it passes cleanly.

## Team Configuration with skillx.toml

For teams that want every member using the same code-review skill, add it to your project's `skillx.toml`:

```toml
[project]
name = "my-project"

[agent]
preferred = "claude-code"
targets = ["claude-code", "cursor", "copilot"]

[skills]
code-review = "github:skillx-run/skillx/examples/skills/code-review"
```

Now any team member can run:

```bash
# Run all skills defined in skillx.toml
skillx run

# Or install persistently
skillx install
```

### Pinning a Version

Pin to a specific commit or tag to prevent unexpected changes:

```toml
[skills]
code-review = "github:skillx-run/skillx/examples/skills/code-review@v1.0.0"
```

### Combining with Other Skills

```toml
[skills]
code-review = "github:skillx-run/skillx/examples/skills/code-review"
testing-guide = "github:skillx-run/skillx/examples/skills/testing-guide"

[skills.dev]
commit-message = "github:skillx-run/skillx/examples/skills/commit-message"
```

The `[skills.dev]` section is for development-only skills that are not needed in CI or production.

## Next Steps

- [Commit Message](/examples/commit-message) — Automate commit message generation
- [Testing Guide](/examples/testing-guide) — Improve test coverage with guided patterns
- [Writing Skills](/guides/writing-skills) — Create a custom review skill for your team's conventions

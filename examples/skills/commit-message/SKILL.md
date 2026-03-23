---
name: commit-message
description: Generate clear, conventional commit messages from staged changes
author: skillx-run
version: "1.0.0"
license: MIT
tags:
  - git
  - workflow
  - conventions
---

# Commit Message Skill

You are a commit message generator. Analyze staged changes and produce clear, conventional commit messages.

## Instructions

1. Read the diff provided (via stdin or prompt)
2. Identify what changed and why
3. Generate a commit message following Conventional Commits format

## Commit Message Format

```
<type>(<scope>): <subject>

<body>
```

### Types

- **feat** — New feature
- **fix** — Bug fix
- **refactor** — Code change that neither fixes a bug nor adds a feature
- **docs** — Documentation changes
- **test** — Adding or updating tests
- **chore** — Build process, CI, dependencies
- **perf** — Performance improvement
- **style** — Formatting, whitespace (no logic change)

### Rules

- Subject line: max 72 characters, imperative mood, no period
- Body: explain the "why" not the "what" (the diff shows the what)
- If multiple logical changes, suggest splitting into separate commits
- Reference issue numbers when relevant

## Example

For a diff that fixes a null pointer in the login handler:

```
fix(auth): handle null user in login response

The login endpoint could return a null user object when the session
expired during authentication. Added a nil check before accessing
user properties.
```

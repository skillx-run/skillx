---
title: Writing Skills
description: How to create Agent Skills — SKILL.md format, frontmatter, directory structure, and best practices.
---

:::tip
See the [examples/skills](https://github.com/skillx-run/skillx/tree/main/examples/skills) directory for complete, runnable example skills you can use as starting points.
:::

## Skill Directory Structure

A skill is a directory with a `SKILL.md` file and optional supporting files:

```
my-skill/
├── SKILL.md          # Required — main instruction file
├── scripts/          # Optional — helper scripts
│   ├── setup.sh
│   └── process.py
└── references/       # Optional — supporting documents
    ├── examples.md
    ├── template.json
    └── schema.sql
```

## SKILL.md Format

The `SKILL.md` file is a Markdown document with optional YAML frontmatter:

```markdown
---
name: code-review
description: Perform structured code reviews with severity levels and actionable feedback
author: skillx-run
version: "1.0.0"
license: MIT
tags: [review, quality, best-practices]
---

# Code Review Skill

You are a code reviewer. Analyze code changes and provide structured, actionable feedback.

## Review Process

1. **Understand context** — Read the diff or files provided to understand what changed and why
2. **Check correctness** — Look for bugs, logic errors, and edge cases
3. **Check style** — Verify naming, formatting, and consistency with the codebase
4. **Check security** — Identify potential vulnerabilities (injection, XSS, secrets, etc.)
5. **Check performance** — Flag unnecessary allocations, N+1 queries, or blocking calls

## Output Format

For each finding, use this format:

    [SEVERITY] file:line — Brief description
      → Suggested fix or explanation

## Summary

End your review with a brief summary:

    ## Summary
    - N critical, N warnings, N suggestions
    - Overall: [Approve / Request Changes / Needs Discussion]
```

### Frontmatter Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | No | Skill display name (defaults to directory name) |
| `description` | string | No | Short description of what the skill does |
| `author` | string | No | Author name or handle |
| `version` | string | No | Skill version (use quotes: `"1.0"`) |
| `tags` | list | No | Tags for discovery and categorization |

All frontmatter fields are optional. The frontmatter block itself is optional — a plain Markdown file works as a skill.

### Frontmatter Parsing

Frontmatter must be delimited by `---` lines at the start of the file:

```markdown
---
name: my-skill
---

Content starts here.
```

If no frontmatter is present, skillx uses the directory name as the skill name and leaves other metadata empty.

## Writing Good Instructions

The body of SKILL.md is what the agent reads and follows. Write it as if you're giving instructions to a skilled developer:

### Be Specific

```markdown
<!-- Bad -->
Process the data.

<!-- Good -->
Read the CSV file provided as input. For each row, validate that
the email column matches RFC 5322 format. Output invalid rows to
`invalid.csv` with a column indicating the validation error.
```

### Structure with Headings

```markdown
## When to Use This Skill

Use this skill when you need to...

## Step-by-Step Instructions

1. First, check that...
2. Then, create...
3. Finally, verify...

## Error Handling

If the input file is missing, report the error and suggest...
```

### Reference Supporting Files

```markdown
## Scripts

Run `scripts/setup.sh` to install dependencies before processing.
The main logic is in `scripts/process.py`.

## References

See `references/style-guide.md` for the coding conventions to follow.
Use `references/template.json` as the base for output files.
```

## Scripts Directory

The `scripts/` directory contains executable helpers that the agent can run:

```
scripts/
├── setup.sh       # Environment setup
├── process.py     # Main processing logic
├── validate.js    # Input validation
└── cleanup.sh     # Cleanup tasks
```

### Supported Script Types

The scanner recognizes these file extensions: `.py`, `.sh`, `.bash`, `.js`, `.ts`, `.rb`, `.pl`, `.ps1`.

### Script Best Practices

1. **Keep scripts focused** — one script per task
2. **Add shebang lines** — `#!/usr/bin/env python3`
3. **Handle errors** — exit with non-zero codes on failure
4. **Avoid side effects** — don't modify files outside the working directory
5. **Document dependencies** — list required tools in SKILL.md

## References Directory

The `references/` directory contains documents, examples, and templates:

```
references/
├── examples.md       # Example inputs and outputs
├── style-guide.md    # Coding conventions
├── schema.sql        # Database schema
├── template.json     # Output template
└── api-spec.yaml     # API specification
```

References are not executed — they are informational files the agent can read for context.

### Avoid Large Files

The scanner flags reference files larger than 50 MB (RS-002). Keep references lean and focused.

## Testing Your Skill

### Scan Before Publishing

```bash
# Check for all findings
skillx scan --fail-on info ./my-skill

# Standard check (default: fail on danger)
skillx scan ./my-skill

# Strict check for publishing
skillx scan --fail-on warn ./my-skill
```

### Test Locally

```bash
# Run with a test prompt
skillx run ./examples/skills/hello-world "Hello"

# Run with a prompt file
skillx run ./examples/skills/code-review -f test-prompt.txt
```

### View Metadata

```bash
skillx info ./my-skill
```

## Avoiding Scanner False Positives

Some scanner rules may trigger on legitimate content:

### MD-003: URLs in documentation

If your SKILL.md references URLs for documentation purposes, this triggers MD-003. Add a comment explaining why:

```markdown
## API Reference

This skill uses the OpenAI API. See https://api.openai.com/docs
for endpoint documentation.
<!-- Note: URL is for documentation reference only, no data is sent -->
```

### SC-006: Legitimate network requests

If a script needs network access, document it clearly:

```markdown
## Network Access

`scripts/fetch.sh` downloads the latest model weights from
the official repository. This is required for the skill to function.
```

## Publishing Checklist

Before sharing your skill:

- [ ] `SKILL.md` has descriptive frontmatter (name, description, author, version)
- [ ] Instructions are clear and specific
- [ ] `skillx scan --fail-on warn` passes
- [ ] Scripts have shebang lines and error handling
- [ ] No unnecessary files in the skill directory
- [ ] No sensitive data (API keys, passwords) in any files
- [ ] References are under 50 MB each

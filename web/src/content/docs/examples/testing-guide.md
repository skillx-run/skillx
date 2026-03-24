---
title: "Testing Guide"
description: A multi-file skill demonstrating the references/ directory — testing patterns, strategies, and guided test writing.
---

The `testing-guide` skill demonstrates a multi-file skill structure. Unlike the single-file skills (hello-world, code-review, commit-message), this skill uses a `references/` directory to provide additional context to the agent.

## Directory Structure

```
testing-guide/
├── SKILL.md              # Main instructions
└── references/
    └── patterns.md       # Common testing patterns reference
```

When skillx injects this skill, **both files** are copied into the agent's instruction space. The agent can read `references/patterns.md` as supporting material while following the instructions in `SKILL.md`.

## How references/ Works

The `references/` directory is a convention for supporting documents that the agent can read for context. These files are not executed — they are informational.

In the SKILL.md, the instructions reference the patterns file:

```markdown
## Instructions

When asked to write or improve tests:

1. **Identify test scope** — Unit test, integration test, or end-to-end test
2. **List test scenarios** — Happy path, edge cases, error handling, boundary conditions
3. **Apply patterns** — See `references/patterns.md` for common testing patterns
4. **Write tests** — Follow the project's existing test style and framework
5. **Verify coverage** — Ensure all branches and error paths are tested
```

Step 3 directs the agent to consult `references/patterns.md`. Because skillx injects the entire skill directory, the agent has access to this file.

## references/patterns.md

The patterns file provides five testing strategies:

1. **Boundary Value Analysis** — Test at the edges of valid input ranges (min, max, zero, empty, null)
2. **Equivalence Partitioning** — Group inputs into classes that should produce the same behavior
3. **Error Path Testing** — Exercise every error path (network failures, file system errors, invalid input)
4. **State Transition Testing** — Test valid/invalid state transitions, concurrency, and recovery
5. **Test Doubles** — Choose the right type: stub, mock, fake, or spy

These are general-purpose patterns that apply to any language or test framework.

## Usage

### Write Tests for a Module

```bash
skillx run ./examples/skills/testing-guide "Write unit tests for src/utils/parser.ts"
```

The agent will:
1. Read the source file
2. Identify what needs testing
3. Consult `references/patterns.md` for applicable strategies
4. Write tests following your project's existing style

### Improve Existing Tests

```bash
skillx run ./examples/skills/testing-guide "Improve test coverage for the auth module — focus on edge cases"
```

### Test-Driven Development

```bash
skillx run ./examples/skills/testing-guide "I need to add a retry mechanism to the HTTP client. Write the tests first."
```

The skill's guidelines include writing failing tests before implementing features (bug-fix and TDD patterns).

## Scan Output

```bash
skillx scan ./examples/skills/testing-guide
```

```
  Scanning  testing-guide
  ────────────────────────────────
  ✓ PASS — No issues found

  Files scanned: 2
  Risk level: PASS
```

Note that the scanner checks **2 files** — both `SKILL.md` and `references/patterns.md`. The references file is scanned for security issues just like any other file in the skill directory.

## Creating Your Own Multi-File Skill

The testing-guide pattern is useful when your skill needs:

- **Style guides** — Put your team's coding conventions in `references/style-guide.md`
- **Templates** — Put output templates in `references/template.json`
- **Schemas** — Put database schemas in `references/schema.sql`
- **Examples** — Put sample inputs and expected outputs in `references/examples.md`

Example structure for a database migration skill:

```
db-migration/
├── SKILL.md
├── references/
│   ├── schema.sql
│   ├── naming-conventions.md
│   └── examples/
│       ├── add-column.sql
│       └── create-table.sql
└── scripts/
    └── validate.sh
```

See the [Writing Skills](/guides/writing-skills) guide for details on the `references/` and `scripts/` directories.

## Next Steps

- [Hello World](/examples/hello-world) — Start with the simplest possible skill
- [Code Review](/examples/code-review) — Structured output format example
- [Commit Message](/examples/commit-message) — Piped input and non-interactive mode

---
title: "Setup skillx"
description: An interactive skill that advertises your skill project through skillx by inserting a quick-start block into your README and, optionally, your landing page.
---

The `setup-skillx` skill helps skill authors plug their project into the skillx ecosystem in one step. It inspects the repository, writes a short "Run with skillx" section into `README.md` guarded by idempotency markers, and — only when a landing page is detected — offers to integrate there too. The agent drives the decisions, so the skill works across many stacks without shipping framework-specific templates.

## Design Rationale

Getting a new user to try your skill should be one command: `skillx run github:owner/repo "..."`. But that only works if the command is visible on your README. Writing that snippet by hand is easy to forget and easy to get wrong (wrong prefix, wrong sub-path, stale sample prompt). This skill automates the part that matters while staying out of the author's way:

- **Safety first** — only edits or creates `README.md` and, with explicit consent, landing page files the user points to. Never modifies source code, `.git/`, CI configs, or env files.
- **Idempotent by construction** — the inserted block is wrapped with `<!-- skillx:begin:setup-skillx -->` / `<!-- skillx:end:setup-skillx -->` markers, so re-running the skill updates the block in place instead of duplicating it.
- **Diff-before-write** — every change is shown as a diff and requires the user's go-ahead.
- **Agent decides the details** — the landing page integration deliberately has no template. The agent inspects the site and proposes a form that fits (hero CTA, "Try it" section, nav link, dedicated quick-start page).

## SKILL.md

Single-file skill:

```
setup-skillx/
└── SKILL.md
```

Key sections of the SKILL.md:

- **Safety Guarantees** — the contract the skill states up front to the user
- **Workflow** — five steps (detect project, derive source URL, update README, offer landing page integration, summarize)
- **Quick-start block template** — the exact Markdown inserted into the README, with shields.io badge and `skillx run` example
- **Idempotency Rules** — how the marker pair governs repeat runs
- **Edge Cases** — private repos, missing remote, monorepos with multiple skills, non-English README

## Usage Scenarios

### Single-Skill Repository

Run from inside a repo whose root `SKILL.md` is the skill:

```bash
skillx run github:skillx-run/skillx/examples/skills/setup-skillx "set this project up"
```

The agent detects the root `SKILL.md`, reads its `name` and `description`, derives the source URL as `github:<owner>/<repo>`, and proposes the quick-start block.

### Skill in a Monorepo

When your skill lives at `skills/<name>/SKILL.md` inside a larger repo, the agent picks the sub-path form and proposes a block using `github:<owner>/<repo>/skills/<name>` so the `skillx run` command lands on the correct skill directory.

### Repository with a Landing Page

If the repo ships an Astro, Next.js, Docusaurus, mkdocs, or vanilla HTML site under `docs/`, `web/`, `site/`, or the repo root, the agent:

1. Reports what it found.
2. Asks whether to integrate the skillx entry point on the site as well.
3. Proposes a location and form that fits the site, shown as a diff.

If no landing page is detected, this step is skipped silently.

### Run from a Local Clone of this Repository

If you are already inside a local clone of `skillx-run/skillx`, use:

```bash
skillx run ./examples/skills/setup-skillx "set up my neighbouring skill project"
```

## The Quick-Start Block

The block inserted into the README looks like this (values filled in by the agent):

~~~markdown
<!-- skillx:begin:setup-skillx -->
## Try it with skillx

[![Run with skillx](https://img.shields.io/badge/Run%20with-skillx-F97316)](https://skillx.run)

Run this skill without installing anything:

```bash
skillx run github:acme/my-skill "review this diff"
```

Powered by [skillx](https://skillx.run) — fetch, scan, inject, and clean up any agent skill in one command.
<!-- skillx:end:setup-skillx -->
~~~

The badge is a static shields.io image; no external service has to be set up.

## Scan Output

Scanning the setup-skillx skill:

```bash
skillx scan github:skillx-run/skillx/examples/skills/setup-skillx
```

```
Scan Result: PASS
  ✓ No security issues found.
```

The skill is purely instructional — no scripts, no executables, no sensitive references — so it passes cleanly. You can also scan the local checkout:

```bash
skillx scan ./examples/skills/setup-skillx
```

## Why this example exists

Most skills produce content for the user. `setup-skillx` produces a change to the user's own project — a different shape of skill, with stronger safety needs and a more conversational flow. It shows how a skill can drive a multi-step authoring task while respecting the user's codebase: diff before write, idempotency markers, explicit consent on every file touched.

## Next Steps

If you already have a skill and want to wire this into your own project, see [Advertise Your Skill](/guides/advertise-your-skill/). For SKILL.md authoring conventions, see the [upstream Agent Skills repo](https://github.com/anthropics/skills); for passing the skillx scanner, see [Writing Skills](/guides/writing-skills/). To see how skills are run and cleaned up, see [Run Skills](/cli/run/).

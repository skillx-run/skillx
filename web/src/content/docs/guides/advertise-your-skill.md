---
title: Advertise Your Skill
description: Add a "Run with skillx" quick-start block to your skill project's README so users can try the skill without installing anything.
---

If you already have a working skill — a directory with a `SKILL.md` somewhere under version control — this page covers the one step that takes it from "readable" to "actually tried": pointing users at a `skillx run` command in your README.

Hand-writing that snippet is easy to get wrong: stale sub-path, wrong platform prefix, mismatched sample prompt. The `setup-skillx` skill writes it for you.

## The One Command

From the root of your skill project:

```bash
skillx run https://github.com/skillx-run/skillx/tree/main/skills/setup-skillx
```

It reads your `SKILL.md` frontmatter, inspects `git remote` to infer the hosting platform and `owner/repo` slug, and proposes a short "Try it with skillx" block for `README.md`. Every change is shown as a diff and requires your confirmation before anything is written.

Want to audit it first without touching files? Add any dry-run cue (`dry run`, `preview only`, `just show me`, `--dry-run`) and the skill will walk through the same detection and show the full diff without writing.

If you already have a local clone of `skillx-run/skillx`, the equivalent local invocation is:

```bash
skillx run ./skills/setup-skillx
```

## What It Touches

- Only `README.md` and localized siblings like `README.zh-CN.md` / `README.ja.md`.
- Landing-page files, **only if you explicitly point it at them** and approve the diff.
- Never modifies source code, `.git/`, lockfiles, CI configs, or env files.
- Never runs installers or any install-type action on your system — the only commands it runs are read-only inspections like `git remote -v`.

## The Inserted Block

The block is wrapped with idempotency markers so a re-run updates it in place instead of duplicating:

~~~markdown
<!-- skillx:begin:setup-skillx -->
## Try it with skillx

[![Run with skillx](https://img.shields.io/badge/Run%20with-skillx-F97316)](https://skillx.run)

Run this skill without installing anything:

```bash
skillx run https://github.com/acme/my-skill "review this diff"
```

Powered by [skillx](https://skillx.run) — fetch, scan, inject, and clean up any agent skill in one command.
<!-- skillx:end:setup-skillx -->
~~~

The badge is a static shields.io image; no external service has to be set up.

Placement preference: immediately after the top-level title, any badge row, and the intro paragraph, before the first `##` section. You can move the marked block anywhere afterwards — re-running the skill updates it in place wherever it lives.

## Common Scenarios

- **Monorepo with multiple skills**: when `SKILL.md` files live under sub-paths, the generated source URL points to that sub-path on the host's web UI (e.g. `https://github.com/org/repo/tree/main/skills/<name>`). If you select more than one skill, the skill emits a single block containing one `skillx run` command per selection.
- **Conversational / wizard-style skill**: if the skill drives its own dialogue (like `setup-skillx` itself) it doesn't need a free-text prompt — the generated command drops the trailing quoted argument, so it looks like `skillx run <url>` on its own.
- **Non-English README**: section heading and prose are localized to the README's language; the command itself and marker comments stay as-is. If you ship multiple language-specific READMEs, each is localized separately.
- **Private repo**: the block still emits — note that users running the skill will need access to the repo.
- **No git remote**: the skill asks you for the canonical repo URL, or falls back to a local-path example.
- **Landing page**: if the repo ships an Astro / Next / Docusaurus / mkdocs site, the skill offers to propose an integration. It does not force a template — it picks a location and form that fits, as a diff.

## After Running

Typical follow-up:

```bash
skillx scan .     # verify your skill still passes the scanner
git add README.md && git commit -m "docs: add skillx quick-start"
```

If the block already exists and nothing changed, the skill tells you and exits — safe to re-run on every release.

## Why this skill exists

Most skills produce content for the user. `setup-skillx` produces a change to the user's own project — a different shape of skill, with stronger safety needs and a more conversational flow. It shows how a skill can drive a multi-step authoring task while respecting the user's codebase: diff before write, idempotency markers, explicit consent on every file touched.

---
title: Advertise Your Skill
description: Add a "Run with skillx" quick-start block to your skill project's README so users can try the skill without installing anything.
---

If you already have a working skill — a directory with a `SKILL.md` somewhere under version control — this page covers the one step that takes it from "readable" to "actually tried": pointing users at a `skillx run` command in your README.

Hand-writing that snippet is easy to get wrong: stale sub-path, wrong platform prefix, mismatched sample prompt. The `setup-skillx` skill writes it for you.

## The One Command

From the root of your skill project:

```bash
skillx run github:skillx-run/skillx/examples/skills/setup-skillx \
  "set this project up"
```

It reads your `SKILL.md` frontmatter, inspects `git remote` to infer the platform and `owner/repo` slug, and proposes a short "Try it with skillx" block for `README.md`. Every change is shown as a diff and requires your confirmation before anything is written.

If you already have a local clone of `skillx-run/skillx`, the equivalent local invocation is:

```bash
skillx run ./examples/skills/setup-skillx "set this project up"
```

## What It Touches

- Only `README.md` and localized siblings like `README.zh-CN.md` / `README.ja.md`.
- Landing-page files, **only if you explicitly point it at them** and approve the diff.
- Never modifies source code, `.git/`, lockfiles, CI configs, or env files.
- Never runs installers or network requests — all changes are local file edits.

## The Inserted Block

The block is wrapped with idempotency markers so a re-run updates it in place instead of duplicating:

~~~markdown
<!-- skillx:begin:setup-skillx -->
## Try it with skillx

[badge] Run this skill without installing anything:

`skillx run <source> "<sample-prompt>"`
<!-- skillx:end:setup-skillx -->
~~~

Placement preference: immediately after the top-level title, any badge row, and the intro paragraph, before the first `##` section. You can move the marked block anywhere afterwards — re-running the skill updates it in place wherever it lives.

## Common Scenarios

- **Monorepo with multiple skills**: when `SKILL.md` files live under sub-paths, the generated source URL includes the sub-path (e.g. `github:org/repo/skills/<name>`). If you select more than one skill, the skill emits a single block containing one `skillx run` command per selection.
- **Non-English README**: section heading and prose are localized to the README's language; the command itself and marker comments stay as-is. If you ship multiple language-specific READMEs, each is localized separately.
- **Private repo**: the block still emits — note that users running the skill will need access to the repo.
- **No git remote**: the skill asks you for the canonical repo URL, or falls back to a local-path example.
- **Landing page**: if the repo ships an Astro / Next / Docusaurus / mkdocs site, the skill offers to propose an integration. It does not force a template — it picks a location and form that fits, as a diff.

See [Setup skillx](/examples/setup-skillx/) for the full workflow and the exact template it inserts.

## After Running

Typical follow-up:

```bash
skillx scan .     # verify your skill still passes the scanner
git add README.md && git commit -m "docs: add skillx quick-start"
```

If the block already exists and nothing changed, the skill tells you and exits — safe to re-run on every release.

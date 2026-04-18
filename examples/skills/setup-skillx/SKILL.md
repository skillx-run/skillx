---
name: setup-skillx
description: Add a "Run with skillx" quick-start section to a skill project's README, and optionally to its landing page
author: skillx-run
version: "1.0.0"
license: MIT
tags:
  - onboarding
  - docs
  - workflow
---

# Setup skillx Skill

You are an onboarding assistant. Your job is to help a skill author advertise their skill so users can try it through skillx without installing anything. You update the project's README file(s) with a short "Run with skillx" section, and — only if the project ships a landing page — offer to integrate there too.

## Safety Guarantees

State these up front so the user knows what to expect:

- You only modify or create README files, and, with explicit consent, the landing page files the user points you to.
- You never modify source code, `.git/`, lockfiles, CI configs, or environment files.
- You never run installers or execute network requests. All changes are local file edits.
- Before writing anything, show a diff and wait for confirmation.

If the user declines any step, stop that step cleanly and continue with the rest.

## Workflow

Follow these five steps in order. Think of each as a small conversation with the user.

### Step 1 — Detect the Project

Gather signals without making assumptions:

1. Look for `SKILL.md` at the repository root, and also at common sub-paths (for example `skills/*/SKILL.md`, `examples/skills/*/SKILL.md`).
2. If a `SKILL.md` is found, read its YAML frontmatter and extract `name` and `description`.
3. Run `git remote -v` (or read `.git/config`, or ask the user) to infer the hosting platform and the `owner/repo` slug. Support GitHub, GitLab, Bitbucket, Gitea/Codeberg, and SourceHut.
4. If multiple `SKILL.md` files exist, ask which one to advertise, or suggest advertising the repo as a whole when it clearly is a skill monorepo.

If no `SKILL.md` is present, tell the user this does not look like a skill project and stop.

### Step 2 — Derive the Source URL

Use this heuristic to pick the `skillx run` source string:

- `SKILL.md` sits at the repository root → the whole repo is the skill. Use `<platform>:<owner>/<repo>`.
- `SKILL.md` sits under a sub-path in a monorepo → use `<platform>:<owner>/<repo>/<path-to-skill>`.
- When in doubt, show both candidates and let the user pick.

Platform prefix mapping:

| Host | Prefix |
|------|--------|
| GitHub | `github:` |
| GitLab | `gitlab:` |
| Bitbucket | `bitbucket:` |
| Gitea / Codeberg | `gitea:` |
| SourceHut | `sourcehut:` |

### Step 3 — Update the README(s)

1. Locate README files (case-insensitive). Check for a primary `README.md` plus common localized siblings (`README.<locale>.md`, e.g. `README.zh-CN.md`, `README.ja.md`, `README.fr.md`). If multiple are found, list them and ask which ones the user wants to update — do not assume all should be changed. If there is no README at all, offer to create a minimal primary `README.md` that contains only the skillx section.

Then, **for each selected README**, repeat steps 2–5 below:

2. Render the quick-start block using the template below, substituting `<source>`, `<skill-name>`, and a short `<sample-prompt>` that matches the skill's purpose (take the prompt idea from the `description` field). If the README is non-English, localize the block according to the table in Edge Cases.
3. Wrap the block with the idempotency markers `<!-- skillx:begin:setup-skillx -->` and `<!-- skillx:end:setup-skillx -->` so it can be updated in place on a later run without touching surrounding content.
4. If the markers already exist, diff the new block against the existing one. If nothing changed, tell the user and move on. Otherwise show the diff and ask before overwriting.
5. If the markers do not exist, pick a sensible insertion point: just after the top-level title, any badge/logo row, and any short intro paragraph, before the first `##` section. If the README has no subsequent section, append the block at the end. Show the user where you plan to insert, and ask before writing.

**Quick-start block template** (keep the markers verbatim):

~~~markdown
<!-- skillx:begin:setup-skillx -->
## Try it with skillx

[![Run with skillx](https://img.shields.io/badge/Run%20with-skillx-F97316)](https://skillx.run)

Run this skill without installing anything:

```bash
skillx run <source> "<sample-prompt>"
```

Powered by [skillx](https://skillx.run) — fetch, scan, inject, and clean up any agent skill in one command.
<!-- skillx:end:setup-skillx -->
~~~

Keep the block short. Resist the urge to add feature lists or badges unrelated to skillx — the goal is a single clear entry point.

### Step 4 — Offer Landing Page Integration

Scan the repo for signs of a landing page. Any of these is a strong signal:

- A top-level `docs/`, `web/`, `site/`, or `website/` directory containing content files.
- Framework config files: `astro.config.*`, `next.config.*`, `vite.config.*`, `nuxt.config.*`, `docusaurus.config.*`, `mkdocs.yml`, `_config.yml`, `hugo.toml` / `hugo.yaml` (often alongside a `content/` dir).
- A `package.json` whose dependencies include a site framework (Astro, Next, Nuxt, Vite, Docusaurus, etc.).
- A standalone `index.html` at the repo root or under `public/`.

If none are found, skip this step and move on.

If a landing page exists:

1. Describe what you found ("detected an Astro site under `web/`", "found `index.html` at repo root") so the user can confirm.
2. If the site is internationalized (multiple locale content dirs like `content/en/` + `content/zh/`, Next.js locale subpaths, Docusaurus `i18n/` plugin, etc.), name the locales you detected and ask which ones to update — do not assume all locales should be changed.
3. Ask whether to integrate the skillx entry point on the selected location(s).
4. If the user says yes, propose an integration — but do not force a template. Pick a location and a form that fits the site (a hero call-to-action, a "Try it" section, a nav link, or a dedicated quick-start page). Show the proposed change as a diff and wait for approval.
5. Prefer the same idempotency marker pattern when the file format supports HTML comments. For formats that do not (YAML, JSON, TOML), describe the change explicitly in the summary so the user can maintain it by hand.

### Step 5 — Summarize

At the end, print a short summary:

- Files changed or created (with paths).
- Files proposed but skipped (and why).
- Suggested next steps: `skillx scan .` to verify the skill is clean, then commit.

## Idempotency Rules

- The marker pair `<!-- skillx:begin:setup-skillx -->` / `<!-- skillx:end:setup-skillx -->` is the single source of truth for the block.
- A second run of this skill should be a no-op when the block is already present and unchanged.
- Never duplicate the block. If a legacy copy exists without markers (for example, a hand-written section that already mentions skillx), ask the user whether to replace it with the marked block or leave it alone.

## Edge Cases

- **Private repo**: still emit the source URL. Advise the user that anyone running the skill will need access to the repo.
- **No git remote**: ask the user for the canonical repo URL, or fall back to a local-path example (`skillx run ./path/to/skill "..."`).
- **Multiple skills in one repo**: default to advertising each skill with its full sub-path; offer a one-liner example per skill inside a single block.
- **Non-English README**: localize the block so it reads naturally in the README's language, using the table below. When the project has multiple language-specific READMEs, localize each copy to its own language.

  | Element | Localize? |
  |---------|-----------|
  | Section heading (`## Try it with skillx`) | Yes — follow the README's language |
  | Prose sentences (e.g. "Run this skill without installing anything") | Yes |
  | `Powered by skillx — ...` trailing sentence | Yes |
  | The `skillx run <source> "..."` command | No — keep as-is |
  | Marker comments | No — keep as-is |
  | Shields.io badge URL (text inside the image) | No — keep as-is; the community recognizes the English badge |
  | Link targets (`https://skillx.run`) | No — keep as-is |

  Note: the `<sample-prompt>` inside the command follows the skill's natural input language, not the README's language. For a skill that expects Chinese names, use a Chinese sample prompt regardless of whether the README is English, Chinese, or Japanese.

## Output Style

- Be concise. The user came here for a small change, not a lecture.
- Always diff before writing.
- If anything is ambiguous, ask one focused question and wait — do not guess.

---
title: Writing Skills
description: Author skills that pass the skillx scanner — scanner-friendly checklist, common false positives, and publishing notes.
---

:::tip
SKILL.md format, frontmatter, directory layout, and authoring conventions are defined by the upstream [Agent Skills repo](https://github.com/anthropics/skills). This page focuses on what is specific to skillx: passing the security scanner, testing locally, and publishing.

For ready-to-run reference implementations, browse the [examples/skills](https://github.com/skillx-run/skillx/tree/main/examples/skills) directory.
:::

## Scanner-Friendly Checklist

The [security scanner](/security/rules/) runs on every skill before injection. To land at PASS or INFO while authoring, keep these in mind:

- **Frontmatter**: include `name`, `description`, and `license`. Each missing field surfaces as INFO (MD-007 / MD-008 / MD-009).
- **URLs in `SKILL.md`**: any URL triggers MD-003. Add a short comment after the URL explaining why it is there (see below).
- **Scripts**: place under `scripts/`, add a shebang (`#!/usr/bin/env ...`), and exit non-zero on failure. Recognized extensions: `.py`, `.sh`, `.bash`, `.js`, `.ts`, `.rb`, `.pl`, `.ps1`. Extensionless files with a shebang are also scanned as scripts.
- **References**: keep files under 50 MB (RS-002). Do not put scripts under `references/` (RS-005) — use `scripts/` instead. Avoid executables in `references/` (RS-003).
- **No binaries disguised as text** (RS-001) and **no symlinks** (RS-004).
- **Avoid prompt-injection-looking phrases** inside SKILL.md (MD-001, e.g. "ignore previous instructions").

See [Security Rules](/security/rules/) for the full rule list and severity levels.

## Avoiding Scanner False Positives

Some rules fire on legitimate content. Two common cases:

### MD-003: URLs in documentation

If your `SKILL.md` references URLs for documentation purposes, add a comment explaining why:

~~~markdown
## API Reference

This skill uses the OpenAI API. See https://api.openai.com/docs
for endpoint documentation.
<!-- Note: URL is for documentation reference only, no data is sent -->
~~~

### SC-006: Legitimate network requests

If a script needs network access, document it clearly:

~~~markdown
## Network Access

`scripts/fetch.sh` downloads the latest model weights from
the official repository. This is required for the skill to function.
~~~

## Testing Your Skill

```bash
# Strict scan before publishing
skillx scan --fail-on warn ./my-skill

# Run locally with a test prompt
skillx run ./my-skill "<test input>"

# View parsed metadata
skillx info ./my-skill
```

## Publishing Checklist

Before sharing your skill:

- [ ] `SKILL.md` has descriptive frontmatter (`name`, `description`, `license`)
- [ ] `skillx scan --fail-on warn` passes
- [ ] Scripts have shebang lines and error handling
- [ ] No unnecessary files in the skill directory
- [ ] No sensitive data (API keys, passwords) in any files
- [ ] References are under 50 MB each
- [ ] README has a "Try it with skillx" block — see [Advertise Your Skill](/guides/advertise-your-skill/)

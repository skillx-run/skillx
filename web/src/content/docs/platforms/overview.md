---
title: Platforms Overview
description: How skillx resolves skill sources and which platforms are supported.
---

## Source Resolution

When you pass a source string to `skillx run` or `skillx scan`, skillx resolves it using this priority order:

### 1. Local Path

Any source that starts with `./`, `/`, `~/`, or points to an existing file system path is treated as a local skill:

```bash
skillx run ./my-skill "prompt"
skillx run /absolute/path/to/skill "prompt"
skillx run ~/skills/my-skill "prompt"
```

Local skills are used directly — no download or caching.

### 2. `github:` Prefix

The shorthand `github:` prefix resolves to a GitHub repository:

```bash
# owner/repo (skill at repo root)
skillx run github:anthropics/skillx-skills "prompt"

# owner/repo/path (skill in a subdirectory)
skillx run github:anthropics/skills/pdf-processing "prompt"

# owner/repo/path@ref (specific branch or tag)
skillx run github:anthropics/skills/pdf-processing@v1.0 "prompt"
```

### 3. GitHub URL

Full GitHub URLs are parsed to extract owner, repo, path, and ref:

```bash
skillx run https://github.com/anthropics/skills/tree/main/pdf-processing "prompt"
```

### 4. Error

If the source doesn't match any of the above, skillx exits with an error:

```
cannot resolve source: 'foobar'. Use a local path (./skill), github: prefix, or GitHub URL
```

## Supported Platforms (v0.1)

| Platform | Status | Source Format |
|----------|--------|---------------|
| Local filesystem | Supported | `./path`, `/path`, `~/path` |
| GitHub | Supported | `github:owner/repo/path` or full URL |
| GitLab | Planned | — |
| Bitbucket | Planned | — |
| Codeberg | Planned | — |
| SourceHut | Planned | — |
| Gitea | Planned | — |
| HuggingFace | Planned | — |

## Caching Behavior

Remote skills are cached locally after download:

- **Cache key**: SHA-256 hash of the full source string
- **Cache location**: `~/.skillx/cache/<hash>/skill-files/`
- **Default TTL**: 24 hours (configurable in `config.toml`)

```toml
# ~/.skillx/config.toml
[cache]
ttl = "7d"        # 7 days
max_size = "2GB"
```

Use `--no-cache` to bypass caching for a single run:

```bash
skillx run --no-cache github:org/skills/my-skill "prompt"
```

Manage the cache with:

```bash
skillx cache ls      # List cached skills
skillx cache clean   # Remove all cached entries
```

## Skill Directory Requirements

Regardless of source, a valid skill directory must contain a `SKILL.md` file at its root. The expected structure is:

```
my-skill/
├── SKILL.md          # Required — main instruction file
├── scripts/          # Optional — helper scripts
│   ├── setup.sh
│   └── process.py
└── references/       # Optional — supporting documents
    ├── examples.md
    └── template.json
```

See [Writing Skills](/guides/writing-skills/) for details on the SKILL.md format.

## Future Platforms

Additional Git hosting platforms and skill directories are planned for future releases. The source resolution system is extensible — new resolvers can be added without changing the core lifecycle.

See:
- [Git Hosts](/platforms/git-hosts/) — details on GitHub and planned Git platforms
- [Skill Directories](/platforms/skill-directories/) — curated registries and marketplaces

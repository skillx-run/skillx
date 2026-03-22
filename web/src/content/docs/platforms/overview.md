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

### 3. `gist:` Prefix

The shorthand `gist:` prefix resolves to a GitHub Gist:

```bash
skillx run gist:abc123 "prompt"
skillx run gist:abc123@revision "prompt"
```

### 4. Platform URL

Full URLs from any supported Git hosting platform are parsed automatically:

```bash
# GitHub
skillx run https://github.com/org/repo/tree/main/path "prompt"

# GitLab
skillx run https://gitlab.com/org/repo/-/tree/main/path "prompt"

# Bitbucket
skillx run https://bitbucket.org/org/repo/src/main/path "prompt"

# Gitea / Codeberg
skillx run https://codeberg.org/user/repo/src/branch/main/path "prompt"

# SourceHut
skillx run https://git.sr.ht/~user/repo/tree/main/item/path "prompt"

# HuggingFace
skillx run https://huggingface.co/user/repo/tree/main/path "prompt"
```

### 5. Archive URL

Direct download links to ZIP or tar.gz archives:

```bash
skillx run https://example.com/skill.zip "prompt"
skillx run https://example.com/skill.tar.gz "prompt"
```

### 6. Skill Directory URL

URLs from 10 supported skill directory platforms:

```bash
skillx run https://skills.sh/pdf-processing "prompt"
skillx run https://skillsmp.com/skills/code-review "prompt"
```

### 7. Error

If the source doesn't match any of the above, skillx exits with an error.

## Supported Platforms

| Platform | Status | Source Format |
|----------|--------|---------------|
| Local filesystem | Supported | `./path`, `/path`, `~/path` |
| GitHub | Supported | `github:owner/repo/path` or URL |
| GitLab | Supported | URL (gitlab.com + self-hosted) |
| Bitbucket | Supported | URL |
| Gitea / Codeberg | Supported | URL (auto-detected + self-hosted) |
| GitHub Gist | Supported | `gist:id` or URL |
| SourceHut | Supported | URL (tarball extraction) |
| HuggingFace | Supported | URL (models/datasets/spaces) |
| Archive | Supported | `.zip` / `.tar.gz` URL |
| Skill Directories | Supported | URL (10 platforms) |

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

See:
- [Git Hosts](/platforms/git-hosts/) — details on all supported Git platforms
- [Skill Directories](/platforms/skill-directories/) — curated registries and marketplaces

---
title: Git Hosts
description: Supported and planned Git hosting platforms for fetching skills.
---

## GitHub (Supported)

GitHub is the primary remote platform in skillx v0.1. Skills can be referenced in two ways.

### `github:` Prefix

The compact shorthand format:

```bash
# Skill at repo root
skillx run github:owner/repo "prompt"

# Skill in a subdirectory
skillx run github:owner/repo/path/to/skill "prompt"

# Specific branch or tag
skillx run github:owner/repo/path@v1.0 "prompt"
skillx run github:owner/repo/path@main "prompt"
```

Format: `github:<owner>/<repo>[/<path>][@<ref>]`

| Component | Required | Description |
|-----------|----------|-------------|
| `owner` | Yes | GitHub user or organization |
| `repo` | Yes | Repository name |
| `path` | No | Subdirectory within the repo containing the skill |
| `ref` | No | Branch, tag, or commit SHA (default: repo default branch) |

### Full URL

Standard GitHub URLs are also parsed:

```bash
skillx run https://github.com/owner/repo "prompt"
skillx run https://github.com/owner/repo/tree/main/path/to/skill "prompt"
```

### How Fetching Works

skillx uses the GitHub REST API to download skill files:

1. Parse the source into owner, repo, optional path, and optional ref
2. Check the local cache for a previous download
3. If not cached (or `--no-cache`), download via the GitHub tarball API
4. Extract the skill directory to `~/.skillx/cache/<hash>/skill-files/`
5. Validate that `SKILL.md` exists in the extracted directory

No GitHub token is required for public repositories. For private repos, set the `GITHUB_TOKEN` environment variable.

### Monorepo Support

Many organizations keep multiple skills in a single repository:

```
github.com/org/skills/
├── pdf-processing/
│   └── SKILL.md
├── code-review/
│   └── SKILL.md
└── data-pipeline/
    └── SKILL.md
```

Reference individual skills using the path component:

```bash
skillx run github:org/skills/pdf-processing "prompt"
skillx run github:org/skills/code-review "prompt"
```

## Planned Platforms

The following platforms are planned for future releases. Each will follow a similar `<platform>:owner/repo/path@ref` pattern.

### GitLab

```bash
# Planned syntax
skillx run gitlab:org/repo/path "prompt"
skillx run https://gitlab.com/org/repo/-/tree/main/path "prompt"
```

Will support both gitlab.com and self-hosted GitLab instances.

### Bitbucket

```bash
# Planned syntax
skillx run bitbucket:org/repo/path "prompt"
skillx run https://bitbucket.org/org/repo/src/main/path "prompt"
```

### Codeberg

```bash
# Planned syntax
skillx run codeberg:user/repo/path "prompt"
skillx run https://codeberg.org/user/repo/src/branch/main/path "prompt"
```

Codeberg uses the Gitea API, so it will share implementation with the Gitea adapter.

### SourceHut

```bash
# Planned syntax
skillx run srht:~user/repo/path "prompt"
skillx run https://git.sr.ht/~user/repo/tree/main/item/path "prompt"
```

### Gitea

```bash
# Planned syntax
skillx run gitea:instance.com/user/repo/path "prompt"
```

Supports self-hosted Gitea instances with configurable base URL.

### HuggingFace

```bash
# Planned syntax
skillx run hf:user/repo/path "prompt"
skillx run https://huggingface.co/user/repo/tree/main/path "prompt"
```

HuggingFace repositories can contain large files via Git LFS. The skillx scanner's RS-002 rule (large file detection) will flag files exceeding 50 MB.

## Authentication

### Public Repositories

No authentication is needed for public repositories on any platform.

### Private Repositories

Set the appropriate environment variable:

| Platform | Environment Variable |
|----------|---------------------|
| GitHub | `GITHUB_TOKEN` |
| GitLab | `GITLAB_TOKEN` (planned) |
| Bitbucket | `BITBUCKET_TOKEN` (planned) |

```bash
export GITHUB_TOKEN=ghp_xxxxxxxxxxxx
skillx run github:private-org/private-skill "prompt"
```

## Caching

All remote sources are cached using a SHA-256 hash of the full source string. The cache is shared across platforms — a skill fetched via `github:org/repo` and `https://github.com/org/repo` will have separate cache entries since the source strings differ.

```bash
skillx cache ls      # View all cached entries
skillx cache clean   # Clear cache
```

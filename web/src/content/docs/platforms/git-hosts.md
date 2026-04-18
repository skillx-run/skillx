---
title: Git Hosts
description: Supported Git hosting platforms for fetching skills.
---

## GitHub (Supported)

GitHub is the primary remote platform. Skills can be referenced in two ways.

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

## GitLab (Supported)

GitLab repositories are supported via URL. Both gitlab.com and self-hosted instances work.

```bash
skillx run https://gitlab.com/org/repo/-/tree/main/path "prompt"
```

Authentication: set `GITLAB_TOKEN` environment variable for private repositories.

GitLab uses the Repository Files API (`/api/v4/projects/:id/repository/tree` and `/repository/files/:path/raw`).

## Bitbucket (Supported)

Bitbucket repositories are supported via URL.

```bash
skillx run https://bitbucket.org/org/repo/src/main/path "prompt"
```

Authentication: set `BITBUCKET_TOKEN` (Bearer) or both `BITBUCKET_USERNAME` and `BITBUCKET_APP_PASSWORD` (Basic Auth).

## Codeberg / Gitea (Supported)

Codeberg (and any Gitea/Forgejo instance) is supported via URL.

```bash
skillx run https://codeberg.org/user/repo/src/branch/main/path "prompt"
```

Authentication: set `GITEA_TOKEN` environment variable.

Self-hosted Gitea instances are auto-detected when the URL contains `/src/branch/` or `/src/tag/` patterns.

## GitHub Gist (Supported)

Gists are supported via both prefix and URL:

```bash
skillx run gist:abc123 "prompt"
skillx run gist:abc123@revision "prompt"
skillx run https://gist.github.com/user/abc123 "prompt"
```

All files in the gist are downloaded and SKILL.md is expected among them.

## Archive Downloads (Supported)

ZIP and tar.gz archives are supported via URL:

```bash
skillx run https://example.com/skill.zip "prompt"
skillx run https://example.com/skill.tar.gz "prompt"
```

Security protections:
- Zip-slip path traversal detection
- Maximum 1000 files per archive
- Maximum 500 MB total uncompressed size
- Single root directory auto-flattening

## SourceHut (Supported)

SourceHut repositories are supported via URL. skillx downloads the repository tarball and extracts the specified sub-path.

```bash
skillx run https://git.sr.ht/~user/repo/tree/main/item/path "prompt"
```

Authentication is not currently required (public repositories only).

## HuggingFace (Supported)

HuggingFace repositories (models, datasets, and spaces) are supported via URL. skillx uses the HuggingFace REST API and auto-detects the repository type.

```bash
# Model repository
skillx run https://huggingface.co/user/model/tree/main/path "prompt"

# Dataset repository
skillx run https://huggingface.co/datasets/user/dataset/tree/main/path "prompt"

# Space repository
skillx run https://huggingface.co/spaces/user/space/tree/main/path "prompt"
```

## Authentication

### Public Repositories

No authentication is needed for public repositories on any platform.

### Private Repositories

Set the appropriate environment variable:

| Platform | Environment Variable |
|----------|---------------------|
| GitHub | `GITHUB_TOKEN` |
| GitLab | `GITLAB_TOKEN` |
| Bitbucket | `BITBUCKET_TOKEN` or `BITBUCKET_USERNAME` + `BITBUCKET_APP_PASSWORD` |
| Gitea/Codeberg | `GITEA_TOKEN` |

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

## Next Steps

- [Platforms Overview](/platforms/overview/) for the full source resolution order
- [Run a Skill](/cli/run/) for CLI flags and runtime behavior
- [Famous Skills](/getting-started/famous-skills/) for real skill examples to try

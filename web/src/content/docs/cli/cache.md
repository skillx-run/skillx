---
title: "skillx cache"
description: Reference for the skillx cache command — manage locally cached skills.
---

## Synopsis

```bash
skillx cache <command>
```

Manage the local skill cache. Skills fetched from remote sources are cached in `~/.skillx/cache/` with a configurable TTL (default 24 hours). Both `run` and `install` share the same cache.

## Subcommands

### `skillx cache ls`

List all currently cached skills.

```bash
$ skillx cache ls

Cached Skills
  pdf-processing — github:skillx-run/skillx/examples/skills/pdf-processing@main (cached 2026-03-22 10:30)
  code-review — github:org/skills/code-review@v2.1 (cached 2026-03-21 14:15)

  2 entries
```

Each entry shows the skill name, source, and when it was cached.

### `skillx cache clean`

Remove all cached skills.

```bash
$ skillx cache clean

✓ Cleaned 2 cached entries.
```

## Cache Behavior

- Remote skills are cached using a SHA-256 hash of the source string as the directory name
- Cache entries include metadata (source, skill name, cache timestamp, TTL)
- Expired entries are automatically skipped when resolving skills
- Use `--no-cache` on `run` or `install` to force a fresh fetch

## Configuration

Cache TTL can be configured in `~/.skillx/config.toml`:

```toml
[cache]
ttl = "24h"      # Cache expiration time (default: 24h)
max_size = "1GB"  # Maximum total cache size
```

## Cache Directory

```
~/.skillx/cache/
└── <source-hash>/
    ├── meta.json       # Cache metadata (source, timestamp, TTL)
    └── <skill-files>   # Cached skill files
```

## Examples

### Check what's cached

```bash
skillx cache ls
```

### Clear all cached skills

```bash
skillx cache clean
```

### Force fresh fetch (bypass cache)

```bash
skillx run github:org/skills/my-skill --no-cache
skillx install github:org/skills/my-skill --no-cache
```

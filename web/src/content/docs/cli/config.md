---
title: "skillx cache & config"
description: Reference for cache management and the config.toml configuration file.
---

## skillx cache

Manage the local skill cache.

### Synopsis

```bash
skillx cache <subcommand>
```

### Subcommands

#### `skillx cache ls`

List all cached skills:

```bash
skillx cache ls
```

```
Cached Skills
  pdf-processing — github:anthropics/skills/pdf-processing (cached 2025-03-15 14:30)
  code-review — github:org/skills/code-review (cached 2025-03-14 09:22)

  2 entries
```

#### `skillx cache clean`

Remove all cached skills:

```bash
skillx cache clean
```

```
✓ Cleaned 2 cached entries.
```

### How Caching Works

When a remote skill is fetched, it is stored in `~/.skillx/cache/` using a SHA-256 hash of the source string as the directory name. On subsequent runs, the cached copy is used if it hasn't exceeded the TTL.

Default cache TTL is 24 hours. Configure it in `~/.skillx/config.toml`:

```toml
[cache]
ttl = "7d"       # 7 days
max_size = "2GB"
```

Use `--no-cache` with `skillx run` to bypass the cache for a single invocation.

## config.toml

skillx reads global configuration from `~/.skillx/config.toml`. All fields are optional — defaults are used for any missing values.

### Full Example

```toml
[cache]
ttl = "24h"           # Cache time-to-live (default: "24h")
max_size = "1GB"       # Max total cache size (default: "1GB")

[scan]
default_fail_on = "danger"  # Default --fail-on level (default: "danger")

[agent.defaults]
preferred = "claude-code"   # Preferred agent when multiple detected
scope = "global"            # Default injection scope (default: "global")

[history]
max_entries = 50       # Max archived sessions to keep (default: 50)
```

### Sections

#### `[cache]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `ttl` | string | `"24h"` | How long cached skills remain valid |
| `max_size` | string | `"1GB"` | Maximum total cache size |

Duration strings support: `s` (seconds), `m` (minutes), `h` (hours), `d` (days).

```toml
[cache]
ttl = "12h"    # 12 hours
ttl = "7d"     # 7 days
ttl = "30m"    # 30 minutes
```

#### `[scan]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `default_fail_on` | string | `"danger"` | Default `--fail-on` level for `skillx scan` |

Valid values: `pass`, `info`, `warn`, `danger`, `block`.

#### `[agent.defaults]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `preferred` | string | — | Agent name to use when multiple are detected |
| `scope` | string | `"global"` | Default injection scope (`global` or `project`) |

#### `[history]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `max_entries` | integer | `50` | Maximum number of archived session manifests to retain |

### Data Directories

```
~/.skillx/
├── config.toml    # This file
├── cache/         # SHA-256 keyed skill cache
│   └── <hash>/
│       └── skill-files/
├── active/        # Active run sessions
│   └── <session-id>/
│       ├── manifest.json
│       ├── skill-files/
│       └── attachments/
└── history/       # Archived session manifests
    └── <session-id>.json
```

### Creating the Config File

The config file is optional. To create one with defaults:

```bash
cat > ~/.skillx/config.toml << 'EOF'
[cache]
ttl = "24h"
max_size = "1GB"

[scan]
default_fail_on = "danger"

[agent.defaults]
scope = "global"

[history]
max_entries = 50
EOF
```

skillx will create the `~/.skillx/` directory automatically on first run if it doesn't exist.

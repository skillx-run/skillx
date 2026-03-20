---
title: config.toml Reference
description: Complete reference for the ~/.skillx/config.toml configuration file.
---

## Location

```
~/.skillx/config.toml
```

The configuration file is optional. If it doesn't exist, all defaults are used. skillx creates the `~/.skillx/` directory automatically on first run.

## Full Default Configuration

```toml
[cache]
ttl = "24h"
max_size = "1GB"

[scan]
default_fail_on = "danger"

[agent.defaults]
# preferred = "claude-code"
scope = "global"

[history]
max_entries = 50
```

## Sections

### [cache]

Controls how fetched remote skills are cached locally.

```toml
[cache]
ttl = "24h"
max_size = "1GB"
```

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `ttl` | string | `"24h"` | Time-to-live for cached entries |
| `max_size` | string | `"1GB"` | Maximum total size of the cache directory |

#### Duration Strings

The `ttl` field accepts human-friendly duration strings:

| Value | Duration |
|-------|----------|
| `"30s"` | 30 seconds |
| `"30m"` | 30 minutes |
| `"24h"` | 24 hours |
| `"7d"` | 7 days |

If the suffix is omitted, seconds are assumed (`"3600"` = 3600 seconds = 1 hour).

#### Cache Location

Cached skills are stored in:

```
~/.skillx/cache/<sha256-hash>/skill-files/
```

The hash is computed from the full source string (e.g., `github:org/repo/path`).

### [scan]

Controls default scan behavior.

```toml
[scan]
default_fail_on = "danger"
```

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `default_fail_on` | string | `"danger"` | Default `--fail-on` level for `skillx scan` |

Valid values: `pass`, `info`, `warn`, `danger`, `block`.

This sets the default when `--fail-on` is not explicitly passed on the command line. The CLI flag always takes precedence:

```bash
# Uses config default (e.g., "danger")
skillx scan ./my-skill

# Overrides config
skillx scan --fail-on warn ./my-skill
```

### [agent.defaults]

Controls agent selection and injection defaults.

```toml
[agent.defaults]
preferred = "claude-code"
scope = "global"
```

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `preferred` | string | — | Preferred agent when multiple are detected |
| `scope` | string | `"global"` | Default injection scope |

#### preferred

When multiple agents are detected and `--agent` is not specified, skillx normally shows an interactive selector. If `preferred` is set and that agent is among the detected agents, it is selected automatically.

Valid values: `claude-code`, `codex`, `copilot`, `cursor`, `universal`.

#### scope

Sets the default injection scope:

- `"global"` — inject to the agent's global skills directory (e.g., `~/.claude/skills/`)
- `"project"` — inject to the project-local directory (e.g., `.claude/skills/`)

The `--scope` CLI flag takes precedence:

```bash
# Uses config default
skillx run ./my-skill "prompt"

# Overrides config
skillx run --scope project ./my-skill "prompt"
```

### [history]

Controls session history retention.

```toml
[history]
max_entries = 50
```

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `max_entries` | integer | `50` | Maximum number of archived session manifests to keep |

Archived sessions are stored in `~/.skillx/history/`. When the count exceeds `max_entries`, the oldest entries are removed.

## Data Directory Layout

```
~/.skillx/
├── config.toml          # This configuration file
├── cache/               # Cached remote skills
│   ├── <hash-1>/
│   │   └── skill-files/
│   │       ├── SKILL.md
│   │       └── scripts/
│   └── <hash-2>/
│       └── skill-files/
├── active/              # Currently running sessions
│   └── <session-id>/
│       ├── manifest.json
│       ├── skill-files/
│       └── attachments/
└── history/             # Archived session manifests
    ├── <session-id-1>.json
    └── <session-id-2>.json
```

## Example Configurations

### Development (relaxed)

```toml
[cache]
ttl = "7d"

[scan]
default_fail_on = "danger"

[agent.defaults]
preferred = "claude-code"
scope = "project"
```

### CI/Production (strict)

```toml
[scan]
default_fail_on = "warn"
```

### Multi-agent workstation

```toml
[agent.defaults]
preferred = "claude-code"
scope = "global"

[cache]
ttl = "24h"
max_size = "2GB"

[history]
max_entries = 100
```

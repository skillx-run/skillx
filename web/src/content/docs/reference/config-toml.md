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

# Custom URL-to-source mappings (optional)
# [[url_patterns]]
# domain = "git.example.com"
# source_type = "gitea"

# Custom agent definitions (optional)
# [[custom_agents]]
# name = "my-agent"
# display_name = "My Agent"
# binary = "my-agent"
# config_dir = ".my-agent"
# lifecycle = "managed_process"
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

Valid values: any agent name from `skillx agents --all` (e.g., `claude-code`, `codex`, `copilot`, `cursor`, `gemini-cli`, `opencode`, `amp`, `windsurf`, `cline`, `roo`, `universal`, or any Tier 3/custom agent name).

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

### [[url_patterns]]

Map custom domains to source types. Useful for self-hosted Git instances.

```toml
[[url_patterns]]
domain = "git.example.com"
source_type = "gitea"

[[url_patterns]]
domain = "gitlab.corp.internal"
source_type = "gitlab"
```

| Key | Type | Description |
|-----|------|-------------|
| `domain` | string | The hostname to match |
| `source_type` | string | Source type: `github`, `gitlab`, `bitbucket`, `gitea`, `sourcehut`, or `huggingface` |

When skillx encounters a URL with a matching domain, it uses the specified source fetcher instead of the default URL recognition logic.

### [[custom_agents]]

Define custom agent adapters without writing Rust code.

```toml
[[custom_agents]]
name = "my-cli-agent"
display_name = "My CLI Agent"
binary = "mycli"
config_dir = ".mycli"
lifecycle = "managed_process"
supports_prompt = true
supports_auto_approve = true
auto_approve_args = ["--yes"]
prompt_flag = "--message"
```

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `name` | string | required | Internal identifier (used with `--agent`) |
| `display_name` | string | auto-capitalized from name | UI display name |
| `binary` | string | — | Binary name for detection and launch |
| `config_dir` | string | required | Config directory name (e.g., `.mycli`) |
| `lifecycle` | string | required | `managed_process` or `file_inject_and_wait` |
| `supports_prompt` | bool | `true` | Whether the agent accepts a prompt argument |
| `supports_auto_approve` | bool | `false` | Whether the agent supports auto-approve mode |
| `auto_approve_args` | list | `[]` | Arguments to pass in auto-approve mode |
| `prompt_flag` | string | — | Flag for passing prompt (e.g., `--message`) |

Custom agents use the same `GenericAdapter` as Tier 3 built-in agents.

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

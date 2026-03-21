---
title: "skillx init"
description: Reference for the skillx init command — initialize a skillx.toml project configuration file.
---

## Synopsis

```bash
skillx init [options]
```

Initialize a new skillx.toml project configuration file in the current directory.

## Options

| Flag | Default | Description |
|------|---------|-------------|
| `--from-installed` | — | Pre-populate with currently installed skills |

## Behavior

- Creates a `skillx.toml` file in the current directory
- Errors if `skillx.toml` already exists
- Default mode creates an empty template with commented examples
- `--from-installed` reads `installed.json` and pre-populates the `[skills]` table with your currently installed skills
- After creation, shows next steps guidance

## Generated File

### Default Template

```toml
[project]
name = ""
description = ""

[agent]
preferred = ""
scope = "global"
targets = []

[skills]
# Add skills here:
# pdf-processing = "github:anthropics/skills/pdf@v1.2"
# code-review = { source = "github:org/skills/cr@v2.1", scope = "project" }

[skills.dev]
# Dev-only skills:
# testing = "github:org/skills/testing"
```

### With --from-installed

When `--from-installed` is used, the `[skills]` table is populated with entries from your installed skills:

```toml
[project]
name = ""
description = ""

[agent]
preferred = ""
scope = "global"
targets = []

[skills]
pdf-processing = "github:anthropics/skills/pdf@v1.2"
formatter = { source = "github:org/skills/cr@v2.1", scope = "project" }

[skills.dev]
```

## Examples

### Create empty skillx.toml

```bash
skillx init
```

### Create from installed skills

```bash
skillx init --from-installed
```

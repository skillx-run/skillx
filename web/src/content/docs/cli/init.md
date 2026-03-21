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
- Default mode creates a minimal TOML structure with empty fields
- `--from-installed` reads `installed.json` and pre-populates the `[skills]` table with your currently installed skills
- After creation, shows next steps guidance

## Generated File

### Default Template

The generated file is a minimal TOML structure with empty fields ready to be filled in:

```toml
[project]

[agent]
scope = "global"
targets = []

[skills]
```

Edit this file to add your project's skills. For example:

```toml
[project]
name = "my-project"
description = "My project description"

[agent]
preferred = "claude-code"
scope = "global"
targets = ["claude-code", "cursor"]

[skills]
pdf-processing = "github:anthropics/skills/pdf@v1.2"
code-review = { source = "github:org/skills/cr@v2.1", scope = "project" }

[skills.dev]
testing = "github:org/skills/testing"
```

### With --from-installed

When `--from-installed` is used, the `[skills]` table is pre-populated with entries from your currently installed skills:

```toml
[project]

[agent]
scope = "global"
targets = []

[skills]
pdf-processing = "github:anthropics/skills/pdf@v1.2"
formatter = "github:org/skills/formatter"
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

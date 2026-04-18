---
title: "skillx install"
description: Reference for the skillx install command — persistently install skills into agent environments.
---

## Synopsis

```bash
skillx install [sources...] [options]
```

Persistently install skills into agent environments. Unlike `skillx run`, installed skills remain in place across sessions and are not cleaned up automatically.

Two modes of operation:

- **Explicit mode**: `skillx install <source1> [source2...]` — install specified skills
- **Manifest mode**: `skillx install` (no args) — install all skills from skillx.toml

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `sources` | No | Skill source(s): local path, `github:`/`gist:` prefix, or URL. If omitted, installs from skillx.toml |

## Options

| Flag | Default | Description |
|------|---------|-------------|
| `--agent <name>` | auto-detect | Target a specific agent (conflicts with `--all`) |
| `--all` | — | Install to all detected agents (conflicts with `--agent`) |
| `--scope <scope>` | `global` | Injection scope: `global` or `project` |
| `--no-cache` | — | Force re-fetch, skip cache |
| `--skip-scan` | — | Skip security scan |
| `--yes` | — | Auto-confirm WARN level risks |
| `--no-save` | — | Don't save to skillx.toml (by default, sources ARE saved) |
| `--dev` | — | Install as dev dependency (saved under `[skills.dev]`) |
| `--prod` | — | Only install production dependencies, skip dev |
| `--prune` | — | Remove installed skills not listed in skillx.toml |

## Install Modes

### Explicit Mode

When one or more source arguments are provided, skillx installs those specific skills:

```bash
skillx install github:skillx-run/skillx/examples/skills/pdf-processing github:org/skills/formatter
```

By default, installed sources are saved to skillx.toml (if it exists). Use `--no-save` to skip saving.

### Manifest Mode

When no source arguments are provided, skillx reads from skillx.toml and installs all listed skills:

```bash
skillx install
```

A summary is shown before installation begins:

```
Found 5 skills (3 new, 1 to update, 1 already installed)
```

## Agent Selection

Agent selection follows this priority:

1. `--agent` flag (explicit single agent)
2. `--all` flag (all detected agents)
3. skillx.toml `[agent].targets` list
4. config.toml `preferred` agent
5. Auto-detection

## Conflict Handling

| Situation | Behavior |
|-----------|----------|
| Active run session exists for the skill | Error — must clean up or wait for the run to finish |
| Skill already installed (same source) | Upgrade to latest version |
| Unmanaged files exist at the injection path | Prompt `Overwrite? [y/N]` |

## Dependency Types

Skills can be categorized as production or dev dependencies:

- **Production** (`[skills]`): Core skills needed for the project
- **Dev** (`[skills.dev]`): Skills only needed during development (e.g., testing, debugging)

Use `--dev` to install as a dev dependency, and `--prod` to skip dev dependencies during manifest mode install.

## Pruning

The `--prune` flag removes installed skills that are not listed in skillx.toml. This is useful for keeping your installed skills in sync with the manifest:

```bash
skillx install --prune
```

Skills with active run sessions are skipped during pruning.

## Examples

### Install a single skill

```bash
skillx install github:skillx-run/skillx/examples/skills/pdf-processing
```

### Install to a specific agent

```bash
skillx install --agent cursor github:org/skills/formatter
```

### Install to all detected agents

```bash
skillx install --all github:org/skills/testing
```

### Install as dev dependency

```bash
skillx install --dev github:org/skills/debug-helper
```

### Install from skillx.toml

```bash
skillx install
```

### Install production only, prune extras

```bash
skillx install --prod --prune
```

### Install without saving to skillx.toml

```bash
skillx install --no-save github:org/skills/one-off-tool
```

## Related Docs

- [Manage Project Skills](/guides/manage-project-skills/)
- [skillx run](/cli/run/)
- [Examples Overview](/examples/overview/)

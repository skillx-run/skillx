---
title: "skillx uninstall"
description: Reference for the skillx uninstall command — remove installed skills from agent environments.
---

## Synopsis

```bash
skillx uninstall <name...> [options]
```

Remove installed skills from agent environments. Supports partial uninstall (from a specific agent) or full removal from all agents.

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `names` | Yes | Skill name(s) to uninstall |

## Options

| Flag | Default | Description |
|------|---------|-------------|
| `--agent <name>` | — | Only remove from a specific agent (partial uninstall) |
| `--keep-in-toml` | — | Keep the entry in skillx.toml |
| `--purge` | — | Also remove cached files |

## Uninstall Behavior

### Full Uninstall (default)

When no `--agent` flag is specified, the skill is fully removed:

1. Injected files are deleted from **all** agents
2. The skill entry is removed from `installed.json`
3. The skill entry is removed from `skillx.toml` (unless `--keep-in-toml`)

### Partial Uninstall

With `--agent`, only the injection for that specific agent is removed:

1. Injected files are deleted from the specified agent only
2. Other agent injections remain intact
3. If this was the last remaining agent, the skill is fully removed (same as full uninstall)

## Options Detail

### --keep-in-toml

Prevents removing the skill entry from skillx.toml. This is useful for temporary removal — the skill can be re-installed later with `skillx install` without needing to re-add the source.

### --purge

In addition to removing injected files, `--purge` also deletes the cached download from `~/.skillx/cache/`. This forces a fresh fetch on the next install or run.

## Examples

### Uninstall a skill

```bash
skillx uninstall pdf-processing
```

### Uninstall from a specific agent only

```bash
skillx uninstall formatter --agent cursor
```

### Uninstall but keep in skillx.toml

```bash
skillx uninstall testing --keep-in-toml
```

### Uninstall and purge cache

```bash
skillx uninstall old-skill --purge
```

### Uninstall multiple skills

```bash
skillx uninstall skill-a skill-b skill-c
```

## Related Docs

- [Manage Project Skills](/guides/manage-project-skills/) for the full cleanup and maintenance workflow around project skills
- [skillx list](/cli/list/) to confirm what is still installed after removal
- [skillx install](/cli/install/) if you are removing temporarily and expect to add the skill back later

---
title: "skillx list"
description: Reference for the skillx list command — list installed skills and check for updates.
---

## Synopsis

```bash
skillx list [options]
```

List all installed skills with their source, version, agents, and scope. Optionally check for available updates.

## Options

| Flag | Default | Description |
|------|---------|-------------|
| `--agent <name>` | — | Filter by agent |
| `--scope <scope>` | `all` | Filter by scope: `project`, `global`, or `all` |
| `--json` | — | Output as JSON (to stdout) |
| `--outdated` | — | Check for available updates |

## Output Formats

### Table (default)

A formatted table is printed to stderr:

```
Installed Skills

  Name              Version   Source                                Agents       Scope
  pdf-processing    v1.2      github:anthropics/skills/pdf@v1.2    claude-code  global
  formatter         v2.1      github:org/skills/cr@v2.1            cursor       project
  testing           latest    github:org/skills/testing             claude-code  global
```

### JSON

Machine-readable output on stdout, suitable for piping to `jq` or other tools:

```bash
skillx list --json
```

```json
[
  {
    "name": "pdf-processing",
    "source": "github:anthropics/skills/pdf@v1.2",
    "version": "v1.2",
    "agents": ["claude-code"],
    "scope": "global"
  }
]
```

JSON output goes to stdout so it can be piped. Status messages still go to stderr.

### Outdated

The `--outdated` flag fetches the latest version for each installed skill and shows a dedicated table of skills with available updates:

```bash
skillx list --outdated
```

```
Outdated Skills

  Name              Installed   Available   Changes
  pdf-processing    v1.2        v1.3        2 files changed
  formatter         v2.1        v2.2        1 file changed
```

## Filtering

### By Agent

Show only skills installed for a specific agent:

```bash
skillx list --agent claude-code
```

### By Scope

Show only skills matching a specific scope:

```bash
skillx list --scope project
```

## Examples

### List all installed skills

```bash
skillx list
```

### List skills for a specific agent

```bash
skillx list --agent claude-code
```

### Output as JSON

```bash
skillx list --json
```

### Check for outdated skills

```bash
skillx list --outdated
```

### Filter by scope

```bash
skillx list --scope project
```

### Pipe JSON to jq

```bash
skillx list --json | jq '.[].name'
```

---
title: Manage Project Skills
description: Move from one-off skillx run usage to a stable project-level workflow with skillx.toml and install lifecycle commands.
---

## From First Run to Project Workflow

`skillx run` is the fastest way to prove a skill is useful. `skillx scan` is where you validate that trust decision more explicitly. This guide is the next stage after both of those: moving from a one-off success to a project setup that teammates, CI, and future sessions can all share.

If `run` answered "does this help?" and `scan` answered "is this acceptable?", the sequence below answers "how do we keep it around cleanly?" The usual order is:

1. Create `skillx.toml` for the project with [`skillx init`](/cli/init/).
2. Add and persist the skills you actually want with [`skillx install`](/cli/install/).
3. Inspect what is currently installed with [`skillx list`](/cli/list/).
4. Refresh installed skills over time with [`skillx update`](/cli/update/).
5. Remove skills you no longer want with [`skillx uninstall`](/cli/uninstall/).

## Recommended Flow

### 1. Initialize the project

Start by creating a manifest in the repository root:

```bash
skillx init
```

If you already tested skills locally and want to capture that state, `skillx init --from-installed` can prefill the manifest from existing installations. See [`skillx init`](/cli/init/).

### 2. Install the skills you want to keep

Use `skillx install` when a skill should remain available instead of being cleaned up after one run:

```bash
skillx install github:skillx-run/skillx/examples/skills/pdf-processing
```

This is the moment you move from ad hoc usage to repeatable project usage. See [`skillx install`](/cli/install/).

### 3. List and audit the current state

Check what is installed, where it is injected, and whether updates are available:

```bash
skillx list
skillx list --outdated
```

Use this as the quick health check for project skills. See [`skillx list`](/cli/list/).

### 4. Update deliberately

When upstream skills change, refresh them with:

```bash
skillx update
```

For safer review, start with `skillx update --dry-run`. See [`skillx update`](/cli/update/).

### 5. Remove what no longer belongs

If a skill is obsolete or was only needed for a temporary workflow:

```bash
skillx uninstall pdf-processing
```

That keeps the manifest and installed state clean. See [`skillx uninstall`](/cli/uninstall/).

## When to Keep Using `run`

Project management commands do not replace `skillx run`. Use [`skillx run`](/cli/run/) for:

- Trying a new skill before deciding to keep it
- Running a one-off task without changing project state
- Comparing agent behavior quickly during evaluation

Once a skill becomes part of normal team workflow, shift to the manifest-driven flow above.

## Where This Fits

- Start with [`skillx run`](/cli/run/) to prove the skill is useful in a real session
- Use [`skillx scan`](/cli/scan/) when you want a clearer security review or CI gate
- Use the commands in this guide when the skill should become part of the project's steady-state setup

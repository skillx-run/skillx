---
title: Troubleshooting
description: Fast answers for the most common install, first-run, scan, and source-resolution problems.
---

## Start Here

If your first run did not work, find the symptom that matches what you saw and use the shortest fix first.

## `skillx: command not found`

The binary is installed, but your shell cannot find it.

### What to check

Run:

```bash
skillx --version
```

If that fails, add the install location to your `PATH`.

### Common fix

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

Then restart your shell or reload your shell config.

If you used the install script, also check `~/.local/bin`.

## `No agents detected`

skillx needs at least one supported agent installed before `run` can launch anything.

### What to check

```bash
skillx agents
```

### Common fix

Install or re-install one supported agent:

- **Claude Code**: `npm install -g @anthropic-ai/claude-code`
- **Codex**: install the Codex CLI and confirm it is on your `PATH`
- **Copilot**: install the GitHub Copilot extension in VS Code
- **Cursor**: install Cursor and open the project once

If no agent is detected, skillx falls back to the universal adapter only when you explicitly use a compatible workflow. For normal first runs, start by getting one real agent detected.

## The scan gate stopped my run

This usually means the skill produced `WARN`, `DANGER`, or `BLOCK` findings before launch.

### What to check

Run the same source through `scan` first:

```bash
skillx scan <source>
```

### How to decide what to do next

- Use `scan` when you want to inspect findings without launching the agent
- Use `detail N` during an interactive `DANGER` prompt if you need the specific finding
- Use `--fail-on warn` in CI or policy-heavy workflows
- Do **not** use `--skip-scan` just to get past the gate unless you already trust the skill and understand the risk

If the skill is blocked because it looks unsafe, the fix is often “choose a different skill” rather than “force it through.”

## The GitHub skill URL did not resolve

This usually means the URL does not point at a real skill directory, or the upstream repository changed.

### What a working URL looks like

```bash
skillx run github:anthropics/skills/skills/frontend-design "prompt"
```

### Common causes

- The URL points at the repo root instead of a skill directory
- The branch or path changed upstream
- The repository is private or no longer available
- You copied a documentation URL instead of a source path

### What to do next

- Compare your link with the curated examples on [Famous Skills](/getting-started/famous-skills/)
- Try the same source with `skillx scan <source>` to isolate resolution from launch behavior
- If this is an external curated skill, verify the upstream GitHub page still exists

## A curated external skill changed or disappeared

Famous Skills are intentionally external, which means upstream authors can rename, move, or remove them.

### What to do

- Re-open the source repository from the [Famous Skills](/getting-started/famous-skills/) page
- Check whether the skill moved to a different path
- If you only need a stable local example, switch to [Official Examples](/examples/overview/)

If a famous skill keeps changing and you need repeatability, prefer installing or pinning a source you control.

## Should I use `run`, `scan`, or `install`?

Use the command that matches your stage:

- Use [`skillx run`](/cli/run/) when you are trying a skill for one task right now
- Use [`skillx scan`](/cli/scan/) when you want the security decision without launching an agent
- Use [`skillx install`](/cli/install/) when a one-off skill becomes something you want to keep around
- Use [Manage Project Skills](/guides/manage-project-skills/) when the skill should become part of normal project workflow

## Still blocked?

If the symptom does not match anything above, the next most useful pages are:

- [Installation](/getting-started/installation/) for install and environment setup
- [First Run](/getting-started/first-run/) for the shortest successful path
- [FAQ](/getting-started/faq/) for the conceptual difference between `run`, `scan`, `install`, and examples
- [Run Skills](/cli/run/) for source resolution and lifecycle details
- [Scan Skills](/cli/scan/) for risk gate and exit-code behavior

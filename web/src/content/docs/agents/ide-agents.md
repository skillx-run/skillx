---
title: IDE Agents
description: How skillx works with GitHub Copilot and Cursor — FileInjectAndWait lifecycle.
---

## Overview

IDE agents run inside editors (VS Code, Cursor) rather than as standalone CLI processes. skillx cannot spawn them directly, so it uses the **FileInjectAndWait** lifecycle:

1. Inject skill files into the agent's expected directory
2. Copy the prompt to the system clipboard
3. Wait for the user to press Enter (signaling completion)
4. Clean up injected files

## GitHub Copilot

[GitHub Copilot](https://github.com/features/copilot) provides AI assistance inside VS Code and other editors.

### Detection

skillx detects Copilot by scanning for the extension directory:

```
~/.vscode/extensions/github.copilot-*
```

If any directory matching the `github.copilot-` prefix exists, Copilot is considered detected.

### Injection Paths

| Scope | Path |
|-------|------|
| Global | `~/.github/skills/<skill-name>/` |
| Project | `.github/skills/<skill-name>/` |

### Launch Workflow

```
1. Skill files copied to injection path
2. Prompt copied to system clipboard
3. Message: "Prompt copied to clipboard. Paste it into Copilot Chat."
4. Message: "Press Enter when done to clean up..."
5. User works with Copilot in VS Code
6. User presses Enter in terminal
7. Cleanup removes injected files
```

### Usage

```bash
skillx run --agent copilot ./my-skill "Explain this codebase"
```

After running, switch to VS Code, open Copilot Chat, and paste the prompt (Cmd+V / Ctrl+V). The skill files are already in place for Copilot to reference.

### YOLO Mode

Not supported. Copilot does not have a flag to skip confirmations.

## Cursor

[Cursor](https://cursor.com) is an AI-first code editor with built-in agent capabilities.

### Detection

skillx detects Cursor through two methods:

1. `cursor` binary in PATH (via `which cursor`)
2. A running Cursor process (checked via system process list)

Either condition is sufficient.

### Injection Paths

| Scope | Path |
|-------|------|
| Global | `~/.cursor/skills/<skill-name>/` |
| Project | `.cursor/skills/<skill-name>/` |

### Launch Workflow

```
1. Skill files copied to injection path
2. Prompt copied to system clipboard
3. Message: "Prompt copied to clipboard. Paste it into Cursor chat."
4. Message: "Press Enter when done to clean up..."
5. User works with Cursor
6. User presses Enter in terminal
7. Cleanup removes injected files
```

### Usage

```bash
skillx run --agent cursor ./my-skill "Refactor the auth module"
```

Switch to Cursor, open the AI chat panel, and paste the prompt. The skill's SKILL.md and supporting files are injected where Cursor can find them.

### YOLO Mode

Not supported. Cursor does not have a CLI flag for autonomous mode.

## Clipboard Integration

Both IDE agents use the system clipboard to transfer the prompt:

- **macOS**: Uses the native `NSPasteboard` API via the `arboard` crate
- **Linux**: Uses `xclip` or `wl-clipboard` (X11/Wayland)
- **Windows**: Uses the native clipboard API

If clipboard access fails (e.g., headless environment), skillx prints a warning but continues. The prompt is not transferred, but the skill files are still injected.

## Process-Free Wait

Since IDE agents are not spawned by skillx, the wait phase works differently:

### Enter Key

skillx blocks on stdin, waiting for the user to press Enter:

```
Press Enter when done to clean up...
```

### Ctrl+C

If the user presses Ctrl+C instead of Enter, skillx catches the signal and proceeds directly to cleanup.

### Timeout

The `--timeout` flag still works with IDE agents:

```bash
skillx run --timeout 1h --agent cursor ./my-skill "prompt"
```

After the timeout, skillx proceeds to cleanup regardless of whether Enter was pressed.

## Comparison

| Feature | Copilot | Cursor |
|---------|---------|--------|
| Lifecycle | FileInjectAndWait | FileInjectAndWait |
| Detection | VS Code extension | Binary or process |
| Prompt delivery | Clipboard | Clipboard |
| YOLO mode | No | No |
| Global inject | `~/.github/skills/` | `~/.cursor/skills/` |
| Project inject | `.github/skills/` | `.cursor/skills/` |

## Tips

### Project Scope for IDE Agents

IDE agents often work best with project-scoped injection since they operate within a project context:

```bash
skillx run --scope project --agent cursor ./my-skill "prompt"
```

This places skill files in the current working directory (e.g., `.cursor/skills/my-skill/`), making them visible in the editor's file tree.

### Keeping Skills Active

If you want to keep a skill available across multiple prompts in a single session, don't press Enter until you're fully done. The skill files remain injected until you signal completion.

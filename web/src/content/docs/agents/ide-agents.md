---
title: IDE Agents
description: How skillx works with GitHub Copilot, Cursor, Windsurf, Cline, and Roo Code — FileInjectAndWait lifecycle.
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

### Auto-approve Mode

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

### Auto-approve Mode

Not supported. Cursor does not have a CLI flag for autonomous mode.

## Clipboard Integration

All IDE agents use the system clipboard to transfer the prompt:

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

## Windsurf

[Windsurf](https://windsurf.com) is an AI-powered code editor.

### Detection

skillx detects Windsurf by checking:

1. `windsurf` binary in PATH
2. A running Windsurf process

### Injection Paths

| Scope | Path |
|-------|------|
| Global | `~/.windsurf/skills/<skill-name>/` |
| Project | `.windsurf/skills/<skill-name>/` |

### Usage

```bash
skillx run --agent windsurf ./my-skill "Refactor the API layer"
```

## Cline

[Cline](https://github.com/cline/cline) is a VS Code extension providing autonomous AI coding capabilities.

### Detection

skillx detects Cline by scanning for the VS Code extension:

```
~/.vscode/extensions/saoudrizwan.claude-dev-*
```

### Injection Paths

| Scope | Path |
|-------|------|
| Global | `~/.cline/skills/<skill-name>/` |
| Project | `.cline/skills/<skill-name>/` |

### Usage

```bash
skillx run --agent cline ./my-skill "Add error handling"
```

## Roo Code

[Roo Code](https://github.com/RooVetGit/Roo-Cline) is a VS Code extension forked from Cline with additional features.

### Detection

skillx detects Roo Code by scanning for the VS Code extension:

```
~/.vscode/extensions/rooveterinaryinc.roo-cline-*
```

### Injection Paths

| Scope | Path |
|-------|------|
| Global | `~/.roo/skills/<skill-name>/` |
| Project | `.roo/skills/<skill-name>/` |

### Usage

```bash
skillx run --agent roo ./my-skill "Implement feature"
```

## Comparison

| Feature | Copilot | Cursor | Windsurf | Cline | Roo Code |
|---------|---------|--------|----------|-------|----------|
| Lifecycle | FileInjectAndWait | FileInjectAndWait | FileInjectAndWait | FileInjectAndWait | FileInjectAndWait |
| Detection | VS Code extension | Binary or process | Binary or process | VS Code extension | VS Code extension |
| Prompt delivery | Clipboard | Clipboard | Clipboard | Clipboard | Clipboard |
| Auto-approve mode | No | No | No | No | No |
| Global inject | `~/.github/skills/` | `~/.cursor/skills/` | `~/.windsurf/skills/` | `~/.cline/skills/` | `~/.roo/skills/` |
| Project inject | `.github/skills/` | `.cursor/skills/` | `.windsurf/skills/` | `.cline/skills/` | `.roo/skills/` |

## Tips

### Project Scope for IDE Agents

IDE agents often work best with project-scoped injection since they operate within a project context:

```bash
skillx run --scope project --agent cursor ./my-skill "prompt"
```

This places skill files in the current working directory (e.g., `.cursor/skills/my-skill/`), making them visible in the editor's file tree.

### Keeping Skills Active

If you want to keep a skill available across multiple prompts in a single session, don't press Enter until you're fully done. The skill files remain injected until you signal completion.

### Skill Discovery

IDE agents discover skills by scanning their respective skills directories. When skillx injects a skill at the project scope (e.g., `.cursor/skills/code-review/`), the agent sees the `SKILL.md` file in its project context and follows the instructions within. Global-scope skills are discovered similarly from the home directory (e.g., `~/.cursor/skills/code-review/`).

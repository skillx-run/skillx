# skillx

> npx for Agent Skills — fetch, scan, inject, run, clean in one command.

**skillx** is a cross-platform CLI tool that provides a zero-install experience for Agent Skills. One command handles the complete lifecycle: fetch → scan → inject → run → clean.

## Quick Start

```bash
# Install
cargo install skillx

# Run a skill (temporary — auto-cleanup after use)
skillx run github:anthropics/skills/pdf-processing "Extract tables from report.pdf"

# Run a local skill
skillx run ./my-skill/ "Do the thing"

# Scan a skill for security risks
skillx scan github:org/skills/data-pipeline

# List detected agents
skillx agents

# View skill metadata
skillx info github:anthropics/skills/pdf-processing

# Manage cache
skillx cache ls
skillx cache clean
```

## Commands

| Command | Description |
|---------|-------------|
| `skillx run <source> [prompt]` | Fetch, scan, inject, and run a skill |
| `skillx scan <source>` | Security scan a skill |
| `skillx agents` | List detected agent environments |
| `skillx info <source>` | Show skill metadata |
| `skillx cache ls\|clean` | Manage local cache |

## Supported Agents (v0.1)

| Agent | Type | YOLO Mode |
|-------|------|-----------|
| Claude Code | CLI | `--dangerously-skip-permissions` |
| OpenAI Codex | CLI | `--full-auto` |
| GitHub Copilot | IDE | — |
| Cursor | IDE | — |

## License

- Code: [Apache-2.0](LICENSE)
- Documentation: [CC-BY-4.0](LICENSE-DOCS)

# skillx

> Run any agent skill. Safely. Without installing it.

<!-- CI badge, crates.io version, license badge will go here -->

**skillx** is a CLI tool that runs Agent Skills without permanently installing them. One command fetches a skill from any Git host, scans it with 23 security rules, injects it into your agent, and cleans everything up when the session ends. No files are left behind.

```bash
skillx run github:anthropics/skills/pdf-processing "Extract tables from report.pdf"
```

## Why skillx?

**No install needed.** Every other skill manager requires permanent installation. skillx runs skills ephemerally by default — fetch, use, auto-clean. When you want persistence, `skillx install` is there, but it's opt-in.

**Security first.** Every skill is scanned before injection. 23 rules across 3 analyzers detect prompt injection, credential access, destructive operations, and disguised binaries. Dangerous patterns are blocked — you never run untrusted code.

**One command.** No config files, no setup. One command handles the entire lifecycle: fetch → scan → inject → run → clean.

## Quick Start

```bash
# Install skillx
curl -fsSL https://skillx.run/install.sh | sh

# Run a skill (temporary — auto-cleans when done)
skillx run github:anthropics/skills/pdf-processing "Extract tables"

# Run a local skill
skillx run ./my-skill "Do the thing"

# Want persistence? Opt in explicitly
skillx install github:anthropics/skills/code-review
```

## Commands

| Command | Description |
|---------|-------------|
| `skillx run <source> [prompt]` | Run a skill temporarily (fetch, scan, inject, run, clean) |
| `skillx install [sources...]` | Install skills persistently (opt-in) |
| `skillx uninstall <name...>` | Remove installed skills |
| `skillx list` | List installed skills |
| `skillx update [names...]` | Update installed skills |
| `skillx init` | Initialize skillx.toml |
| `skillx scan <source>` | Security scan a skill |
| `skillx agents` | List detected agent environments |
| `skillx info <source>` | Show skill metadata |
| `skillx cache ls\|clean` | Manage local cache |

## Supported Agents

32 built-in agents across 3 tiers, plus custom agent support:

- **Tier 1**: Claude Code, Codex, GitHub Copilot, Cursor
- **Tier 2**: Gemini CLI, OpenCode, Amp, Windsurf, Cline, Roo
- **Tier 3**: 21 additional agents via generic adapter
- **Custom agents** via `config.toml`

## Supported Sources

10 source types with smart URL recognition for 20+ platforms:

| Source | Example |
|--------|---------|
| Local | `./my-skill/` |
| GitHub | `github:org/repo/path@ref` |
| GitLab | `https://gitlab.com/org/repo/-/blob/main/skill.md` |
| Bitbucket | `https://bitbucket.org/org/repo/src/main/skill.md` |
| Gitea / Codeberg | `https://codeberg.org/org/repo/src/branch/main/skill.md` |
| Gist | `gist:username/id` |
| SourceHut | `https://git.sr.ht/~user/repo/tree/main/item/skill.md` |
| HuggingFace | `https://huggingface.co/org/model/blob/main/skill.md` |
| Archive | `https://example.com/skill.tar.gz` |
| Skill Directories | 10 supported platforms |

## Security

23 rules across 3 analyzers (markdown, script, resource):

- **5 risk levels**: Pass → Info → Warn → Danger → Block
- **Interactive gating**: auto-pass for Pass/Info, prompt for Warn, confirmation for Danger, refuse for Block
- **SARIF 2.1.0** output for CI integration

```bash
skillx scan github:org/skills/data-pipeline
skillx scan --format sarif ./my-skill/
```

## Project Configuration

For teams that want reproducible skill sets, define them in `skillx.toml`:

```toml
[project]
name = "my-project"

[agent]
preferred = "claude-code"

[skills]
pdf-processing = "github:anthropics/skills/pdf@v1.2"
code-review = { source = "github:org/skills/cr@v2.1", scope = "project" }
```

## Documentation

Full documentation at [https://skillx.run](https://skillx.run).

## License

- Code: [Apache-2.0](LICENSE)
- Documentation: [CC-BY-4.0](LICENSE-DOCS)

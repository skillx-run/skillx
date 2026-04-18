# skillx

> Run any agent skill. Safely. Without installing it.

[![CI](https://github.com/skillx-run/skillx/actions/workflows/ci.yml/badge.svg)](https://github.com/skillx-run/skillx/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/skillx.svg)](https://crates.io/crates/skillx)
[![Downloads](https://img.shields.io/crates/d/skillx.svg)](https://crates.io/crates/skillx)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Docs](https://img.shields.io/badge/docs-skillx.run-brightgreen.svg)](https://skillx.run)

**skillx** is a CLI tool that runs Agent Skills without permanently installing them. One command fetches a skill from Git hosts and other supported sources, scans it for risky patterns, injects it into your agent, and cleans everything up when the session ends. No files are left behind.

```bash
skillx run ./examples/skills/name-poem "Your Name"
```

## Why skillx?

**No install needed.** Every other skill manager requires permanent installation. skillx runs skills ephemerally by default — fetch, use, auto-clean. When you want persistence, `skillx install` is there, but it's opt-in.

**Security first.** Every skill is scanned before injection. Built-in analyzers look for prompt injection, credential access, destructive operations, and disguised binaries before launch. Dangerous patterns are blocked — you never run untrusted code silently.

**One command.** No config files, no setup. One command handles the entire lifecycle: fetch → scan → inject → run → clean.

## Quick Start

```bash
# Install skillx
curl -fsSL https://skillx.run/install.sh | sh

# Run a local skill (temporary — auto-cleans when done)
skillx run ./examples/skills/name-poem "Your Name"

# Run a skill from GitHub
skillx run github:skillx-run/skillx/examples/skills/name-poem "Your Name"

# Want persistence? Opt in explicitly
skillx install github:skillx-run/skillx/examples/skills/name-poem
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

Built-in agents across multiple tiers, plus custom agent support:

- **Tier 1**: Claude Code, Codex, GitHub Copilot, Cursor
- **Tier 2**: Gemini CLI, OpenCode, Amp, Windsurf, Cline, Roo
- **Tier 3**: 21 additional agents via generic adapter
- **Custom agents** via `config.toml`
- **Universal fallback** for unrecognized environments

## Supported Sources

skillx is built around local paths, Git hosts, archives, and other compatible source URLs:

| Source | Example |
|--------|---------|
| Local | `./my-skill/` |
| GitHub | `github:org/repo/path@ref` |
| GitLab | `https://gitlab.com/org/repo/-/blob/main/skill.md` |
| Bitbucket | `https://bitbucket.org/org/repo/src/main/skill.md` |
| Gitea / Forgejo / Codeberg | `https://codeberg.org/org/repo/src/branch/main/skill.md` |
| Gist | `gist:username/id` |
| SourceHut | `https://git.sr.ht/~user/repo/tree/main/item/skill.md` |
| HuggingFace | `https://huggingface.co/org/model/blob/main/skill.md` |
| Archive | `https://example.com/skill.tar.gz` |

skillx also keeps compatibility with selected legacy directory links when they resolve cleanly to underlying Git repositories, but those URLs are not the recommended discovery path.

Custom URL patterns can be added via `config.toml` `[[url_patterns]]`.

## Security

Built-in analyzers cover Markdown, scripts, and bundled resources:

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
name-poem = "github:skillx-run/skillx/examples/skills/name-poem"
code-review = { source = "github:skillx-run/skillx/examples/skills/code-review", scope = "project" }
```

```bash
skillx init                  # Create skillx.toml
skillx init --from-installed # Create from currently installed skills
```

## Examples

Browse the [examples/skills](examples/skills) directory for complete, runnable example skills including name-poem, hello-world, code-review, testing-guide, commit-message, and setup-skillx.

## Documentation

Full documentation at [https://skillx.run](https://skillx.run).

## License

- Code: [Apache-2.0](LICENSE)
- Documentation: [CC-BY-4.0](LICENSE-DOCS)

# skillx

> npx for Agent Skills — fetch, scan, inject, run, clean in one command.

<!-- CI badge, crates.io version, license badge will go here -->

**skillx** is a cross-platform CLI tool that provides a zero-install experience for Agent Skills. One command handles the complete lifecycle: fetch, scan, inject, run, clean. It supports 32 built-in agents, 10 source types, persistent installs, and project-level configuration via `skillx.toml`.

## Quick Start

```bash
# Install via Homebrew (macOS / Linux)
brew install skillx-run/tap/skillx

# Or install via Cargo
cargo install skillx

# Run a skill temporarily
skillx run github:anthropics/skills/pdf-processing "Extract tables from report.pdf"

# Install a skill persistently
skillx install github:anthropics/skills/code-review

# Install all skills from skillx.toml
skillx install

# List installed skills
skillx list

# Update all skills
skillx update
```

## Commands

| Command | Description |
|---------|-------------|
| `skillx run <source> [prompt]` | Fetch, scan, inject, and run a skill (temporary) |
| `skillx install [sources...]` | Install skills persistently (explicit or from skillx.toml) |
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

- **Tier 1** (full support): Claude Code, Codex, GitHub Copilot, Cursor
- **Tier 2** (tested): Gemini CLI, OpenCode, Amp, Windsurf, Cline, Roo
- **Tier 3** (community): 21 additional agents via generic adapter
- **Custom agents** via `config.toml` `[[custom_agents]]`
- **Universal fallback** for unrecognized environments

```bash
skillx agents        # List detected agents
skillx agents --all  # List all 32 agents
```

## Supported Sources

10 source types with URL smart recognition for 20+ platforms:

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
| Skill Directories | 10 supported directory platforms |

Custom URL patterns can be added via `config.toml` `[[url_patterns]]`.

## Security

Built-in security scanner with 21 rules across 3 analyzers (markdown, script, resource):

- **5 risk levels**: Pass, Info, Warn, Danger, Block
- **Interactive gating**: auto-pass for Pass/Info, prompt for Warn, confirmation for Danger, refuse for Block
- **SARIF 2.1.0** output for CI integration

```bash
skillx scan github:org/skills/data-pipeline
skillx scan --format sarif ./my-skill/
```

## Project Configuration

Define project skills in `skillx.toml`:

```toml
[project]
name = "my-project"

[agent]
preferred = "claude-code"

[skills]
pdf-processing = "github:anthropics/skills/pdf@v1.2"
code-review = { source = "github:org/skills/cr@v2.1", scope = "project" }
```

```bash
skillx init              # Create skillx.toml
skillx init --from-installed  # Create from currently installed skills
```

## Documentation

Full documentation is available at [https://skillx.run](https://skillx.run).

## License

- Code: [Apache-2.0](LICENSE)
- Documentation: [CC-BY-4.0](LICENSE-DOCS)

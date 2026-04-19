# skillx

> Run any agent skill. Safely. Without installing it.

[![CI](https://github.com/skillx-run/skillx/actions/workflows/ci.yml/badge.svg)](https://github.com/skillx-run/skillx/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/skillx.svg)](https://crates.io/crates/skillx)
[![Downloads](https://img.shields.io/crates/d/skillx.svg)](https://crates.io/crates/skillx)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Docs](https://img.shields.io/badge/docs-skillx.run-brightgreen.svg)](https://skillx.run)

**skillx** is a CLI tool that runs Agent Skills without permanently installing them. One command fetches a skill from Git hosts and other supported sources, scans it for risky patterns, injects it into your agent, and cleans everything up when the session ends. No files are left behind.

```bash
skillx run github:skillx-run/skillx/examples/skills/name-poem "Ada Lovelace"
```

## Why skillx?

- **No install needed.** Tools like `skills` and `skillfish` drop files into your agent and leave them there. skillx runs skills ephemerally by default — fetch, use, auto-clean. When you want persistence, `skillx install` is there, but it's opt-in.
- **Security first.** Every skill is scanned before injection. 31 built-in rules look for prompt injection, credential access, destructive operations, and disguised binaries. Dangerous patterns are blocked — you never run untrusted code silently.
- **One command.** No config files, no setup. `skillx run` handles the entire lifecycle: fetch → scan → inject → run → clean.

## Install

```bash
# Shell installer (macOS / Linux)
curl -fsSL https://skillx.run/install.sh | sh

# Homebrew (macOS / Linux)
brew install skillx-run/tap/skillx

# Cargo (any platform with Rust toolchain)
cargo install skillx

# Cargo binstall (prebuilt binaries, no build step)
cargo binstall skillx
```

Windows binaries are on the [Releases page](https://github.com/skillx-run/skillx/releases).

## Quick Start

```bash
# Run a skill from GitHub (temporary — auto-cleans when done)
skillx run github:skillx-run/skillx/examples/skills/name-poem "Ada Lovelace"

# Paste a full URL — skillx recognizes 20+ platforms
skillx run https://github.com/skillx-run/skillx/tree/main/examples/skills/code-review

# Scan a skill before running
skillx scan github:skillx-run/skillx/examples/skills/name-poem

# Want persistence? Opt in explicitly
skillx install github:skillx-run/skillx/examples/skills/name-poem
```

Browse [`examples/skills/`](examples/skills) for runnable demos (`name-poem`, `hello-world`, `code-review`, `testing-guide`, `commit-message`) and [`skills/`](skills) for first-party skills (currently `setup-skillx`).

## How It Works

```
┌───────┐   ┌──────┐   ┌────────┐   ┌─────┐   ┌───────┐
│ fetch │ → │ scan │ → │ inject │ → │ run │ → │ clean │
└───────┘   └──────┘   └────────┘   └─────┘   └───────┘
```

1. **Fetch** — Resolves the source (local path, `github:`, full URL, archive, …) and downloads with caching.
2. **Scan** — Runs 31 security rules across Markdown, scripts, and bundled resources.
3. **Inject** — Copies or inlines skill files into the detected agent's expected location (e.g. `.claude/skills/`, `.cursor/skills/`).
4. **Run** — Launches the agent with the skill invocation prefix (e.g. `/name-poem`) prepended to your prompt.
5. **Clean** — Removes injected files and archives the session manifest to `~/.skillx/history/`.

## vs. install-based skill managers

Tools like `skills` and `skillfish` drop skill files into your agent and leave them there. skillx fetches, scans, runs once, and cleans up.

**Install-based managers — 4 steps per skill:**

1. Install the skill onto your machine
2. Link it into your agent's folder
3. Audit the files yourself (nothing scans before injection)
4. Uninstall it when you're done

**skillx — 1 command:**

```bash
skillx run <url> "prompt"
```

skillx does the rest: scan → inject → run → clean. Persistent installation is still available via `skillx install`, but it's opt-in.

## Commands

**Core**

| Command | Description |
|---------|-------------|
| `skillx run <source> [prompt]` | Run a skill temporarily (fetch, scan, inject, run, clean) |
| `skillx install [sources...]`  | Install skills persistently (opt-in) |

**Manage installed skills**

| Command | Description |
|---------|-------------|
| `skillx list`                | List installed skills |
| `skillx update [names...]`   | Update installed skills |
| `skillx uninstall <name...>` | Remove installed skills |
| `skillx init`                | Initialize `skillx.toml` |

**Utilities**

| Command | Description |
|---------|-------------|
| `skillx scan <source>`     | Security scan a skill |
| `skillx info <source>`     | Show skill metadata |
| `skillx agents`            | List detected agent environments |
| `skillx cache ls\|clean`   | Manage local cache |
| `skillx upgrade`           | Check for and upgrade skillx itself |

## Supported Agents

32 built-in agents plus custom support:

- **Tier 1**: Claude Code, Codex, GitHub Copilot, Cursor
- **Tier 2**: Gemini CLI, OpenCode, Amp, Windsurf, Cline, Roo
- **Tier 3**: 21 additional agents via a data-driven generic adapter
- **Custom agents** via `[[custom_agents]]` in `~/.skillx/config.toml`
- **Universal fallback** for unrecognized environments

Run `skillx agents --all` to list every built-in adapter.

## Supported Sources

skillx recognizes local paths, shorthand prefixes, and full URLs across Git hosts and other sources:

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

skillx also keeps compatibility with selected skill-directory sites when they resolve cleanly to underlying Git repositories. Custom URL patterns can be added via `[[url_patterns]]` in `config.toml`.

## Security

Three built-in analyzers, 31 rules total, 5 risk levels:

- **Markdown analyzer** — prompt injection, sensitive directory access, hidden zero-width text, disguised data URIs, missing frontmatter fields.
- **Script analyzer** — `eval`, `rm -rf`, base64 / hex decode, environment-variable exfiltration, binary blobs masquerading as text.
- **Resource analyzer** — disguised file types, oversized files, executable files under `references/`, symlinks anywhere in the skill tree (never followed).

Risk levels gate the run interactively:

| Level | Behavior |
|-------|----------|
| Pass / Info | Auto-pass |
| Warn   | Prompt (`y/n`) |
| Danger | Confirmation required (`yes` + acknowledge details) |
| Block  | Refuses to run |

```bash
skillx scan github:org/skills/data-pipeline          # Text report
skillx scan --format sarif ./my-skill/               # SARIF 2.1.0 for CI
skillx scan --format json ./my-skill/                # JSON for scripting
```

A GitHub Action (`.github/actions/scan/action.yml`) is available for CI. See [SECURITY.md](SECURITY.md) for reporting vulnerabilities.

## Project Configuration

For teams that want reproducible skill sets, declare them in `skillx.toml`:

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
skillx run                   # Run all skills declared in skillx.toml
```

## Documentation & Community

- **Full docs**: [https://skillx.run](https://skillx.run)
- **Changelog**: [CHANGELOG.md](CHANGELOG.md)
- **Contributing**: [CONTRIBUTING.md](CONTRIBUTING.md)
- **Security reporting**: [SECURITY.md](SECURITY.md)

## License

Code under [Apache-2.0](LICENSE); documentation under [CC-BY-4.0](LICENSE-DOCS).

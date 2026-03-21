# skillx Project Guide

## Architecture

Monorepo with three components:

- `cli/` — Rust CLI tool (workspace member)
- `web/` — Astro + Starlight site (landing + docs + blog)
- `registry/` — Cloudflare Workers API (v0.4+, placeholder)

### CLI Structure

`cli/src/lib.rs` owns all business logic. `cli/src/main.rs` is a thin shell (clap parse → call lib).
Integration tests access internals via `use skillx::...`.

Key modules:
- `source/` — Skill fetching from multiple platforms. Resolve priority: local path > `github:`/`gist:` prefix > URL > error
  - `url.rs` — URL smart recognition engine (20+ platforms)
  - `url_patterns.rs` — Domain-to-source-type mappings (built-in + custom via config.toml)
  - `resolver.rs` — Unified resolve + fetch + cache abstraction (requires `&Config` param)
  - `github.rs` — GitHub Contents API
  - `gitlab.rs` — GitLab Repository Files API (supports self-hosted)
  - `bitbucket.rs` — Bitbucket Source API
  - `gitea.rs` — Gitea/Forgejo/Codeberg Contents API (supports self-hosted)
  - `gist.rs` — GitHub Gist API
  - `sourcehut.rs` — SourceHut tarball download + sub-path extraction
  - `huggingface.rs` — HuggingFace REST API (models/datasets/spaces type inference)
  - `archive.rs` — ZIP/tar.gz download + extraction (with zip-slip protection)
  - `skills_directory.rs` — Skills directory platform HTML parsing (10 platforms)
  - `local.rs` — Local filesystem source
- `scanner/` — Security scanning with 5 risk levels (Pass/Info/Warn/Danger/Block)
  - `markdown_analyzer.rs` — MD-001~006 (prompt injection, sensitive dirs, etc.)
  - `script_analyzer.rs` — SC-001~011 (binary detection, eval, rm -rf, etc.)
  - `resource_analyzer.rs` — RS-001~003 (disguised files, large files, executable in refs)
  - `rules.rs` — All regex patterns (use `r#"..."#` format for Rust 2021 compat)
  - `report.rs` — Text, JSON, and SARIF 2.1.0 output formatters
- `agent/` — Agent detection & adapters (AgentAdapter trait with async_trait)
  - Tier 1: claude-code, codex, copilot, cursor
  - Tier 2: gemini-cli, opencode, amp, windsurf, cline, roo
  - Tier 3: 21 agents via `generic.rs` (AgentDef + GenericAdapter, data-driven)
  - User custom agents from config.toml `[[custom_agents]]` (also via GenericAdapter)
  - universal (fallback, always last in registry)
- `session/` — Session lifecycle, manifest, inject, cleanup
  - `inject.rs` — `inject_and_collect()` (core) + `inject_skill()` (manifest wrapper)
  - Signal handling via `tokio::signal::ctrl_c()` + `tokio::select!`
  - Interactive orphaned session recovery with metadata display
- `gate.rs` — Scan result gating (PASS/INFO auto-pass, WARN prompt, DANGER interactive, BLOCK refuse)
- `installed.rs` — Persistent install state (`~/.skillx/installed.json`)
- `cache.rs` — Cache management (SHA256 source hash, TTL)
- `config.rs` — `~/.skillx/config.toml` handling (incl. `[[url_patterns]]`, `[[custom_agents]]`)
- `project_config.rs` — `skillx.toml` project-level configuration ([skills] table format)
- `types.rs` — Shared types (Scope enum)
- `error.rs` — SkillxError (thiserror) + Result alias
- `ui.rs` — Terminal output helpers (console + indicatif)
- `commands/` — Command implementations (10 commands, anyhow::Result)
  - `run.rs` — Temporary run (inject → launch → cleanup, skips inject if already installed)
  - `install.rs` — Persistent install (explicit sources or from skillx.toml)
  - `uninstall.rs` — Remove installed skills (per-agent partial or full)
  - `list.rs` — List installed skills (table/JSON, --outdated check)
  - `update.rs` — Update installed skills (SHA256 diff, --dry-run)
  - `init.rs` — Initialize skillx.toml (empty or --from-installed)
  - `scan.rs`, `agents.rs`, `info.rs`, `cache.rs`

### Error Strategy

- `thiserror` (`SkillxError`): library modules
- `anyhow`: command layer (`commands/*.rs`)
- `main.rs`: catches anyhow::Error, formats via `ui::error()`

### Run Command Lifecycle

1. Load Config + ProjectConfig (skillx.toml)
2. Resolve source(s) — CLI arg or skillx.toml `[skills]`
3. Scan each skill (unless --skip-scan)
4. Gate via `gate::gate_scan_result()` (PASS/INFO auto-pass, WARN Y/n, DANGER `yes`+`detail N`, BLOCK refuse)
5. Detect Agent (CLI --agent > skillx.toml agent.preferred > config preferred > auto-detect)
6. Check installed state — skip inject/cleanup if already installed
7. Inject all skills (copy files + SHA256 + manifest)
8. Launch (CLI: subprocess, IDE: clipboard + wait)
9. Wait (with Ctrl+C and --timeout support)
10. Cleanup (remove injected files, archive session) — skipped for installed skills

### skillx.toml Format

```toml
[project]
name = "my-project"
description = "..."

[agent]
preferred = "claude-code"
scope = "project"
targets = ["claude-code", "cursor"]

[skills]
pdf-processing = "github:anthropics/skills/pdf@v1.2"
code-review = { source = "github:org/skills/cr@v2.1", scope = "project" }

[skills.dev]
testing = "github:org/skills/testing"
```

SkillValue supports string shorthand (`"source"`) and detailed object (`{ source, scope, skip_scan }`).

## Build & Test

```bash
cargo build --workspace          # Build all
cargo test --workspace           # Run all tests (212+)
cargo build --release            # Release build
cargo run -- run ./skill "msg"   # Run CLI
cargo run -- run                 # Run from skillx.toml
cargo run -- install ./skill     # Install persistently
cargo run -- install             # Install from skillx.toml
cargo run -- uninstall my-skill  # Uninstall
cargo run -- list                # List installed
cargo run -- list --json         # JSON output
cargo run -- update              # Update all
cargo run -- update --dry-run    # Check for updates
cargo run -- init                # Create skillx.toml
cargo run -- scan ./skill        # Scan skill
cargo run -- agents              # List agents
cargo run -- agents --all        # List all 32 agents
cargo run -- info ./skill        # Show info
cargo run -- cache ls            # List cache
```

## Conventions

- Code, comments, docs, commits in English
- Frequent atomic commits
- Test fixtures in `cli/tests/fixtures/`
- v0.1 scanner uses regex (not tree-sitter)
- Agent adapters implement `AgentAdapter` trait with `async_trait` (32 built-in + custom)
- Source fetchers use FetchContext structs to avoid excessive arguments
- Regex patterns in rules.rs must use `r#"..."#` (Rust 2021 raw strings)
- All user-facing output goes to stderr (via `eprintln!` / `ui::*`)
- JSON output goes to stdout (for piping)
- `resolve_and_fetch()` and `AgentRegistry::new()` require `&Config` parameter
- config.toml supports `[[url_patterns]]` and `[[custom_agents]]`
- `skillx.toml` uses `[skills]` table format (not `[[skills]]` array)
- SkillSource has 10 variants: Local, GitHub, GitLab, Bitbucket, Gitea, Gist, SourceHut, HuggingFace, Archive, SkillsDirectory

## Data Directories

```
~/.skillx/
├── config.toml      # Global config (url_patterns, custom_agents, cache, scan, agent, history)
├── installed.json   # Persistent install state (skills, injections, SHA256)
├── cache/           # Cached skills (TTL-based)
├── active/          # Active run sessions
└── history/         # Archived session manifests
```

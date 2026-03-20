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
  - `url_patterns.rs` — Domain-to-source-type mappings
  - `resolver.rs` — Unified resolve + fetch + cache abstraction
  - `github.rs` — GitHub Contents API
  - `gitlab.rs` — GitLab Repository Files API (supports self-hosted)
  - `bitbucket.rs` — Bitbucket Source API
  - `gitea.rs` — Gitea/Forgejo/Codeberg Contents API (supports self-hosted)
  - `gist.rs` — GitHub Gist API
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
  - universal (fallback, always last in registry)
- `session/` — Session lifecycle, manifest, inject, cleanup
  - Signal handling via `tokio::signal::ctrl_c()` + `tokio::select!`
  - Interactive orphaned session recovery with metadata display
- `cache.rs` — Cache management (SHA256 source hash, TTL)
- `config.rs` — `~/.skillx/config.toml` handling
- `types.rs` — Shared types (Scope enum)
- `error.rs` — SkillxError (thiserror) + Result alias
- `ui.rs` — Terminal output helpers (console + indicatif)
- `commands/` — Command implementations using resolver (anyhow::Result)

### Error Strategy

- `thiserror` (`SkillxError`): library modules
- `anyhow`: command layer (`commands/*.rs`)
- `main.rs`: catches anyhow::Error, formats via `ui::error()`

### Run Command Lifecycle

1. Resolve (source → resolver → local/GitHub/GitLab/Bitbucket/Gitea/Gist/Archive/SkillsDirectory/cache)
2. Scan (unless --skip-scan)
3. Gate (PASS/INFO auto-pass, WARN Y/n, DANGER `yes`+`detail N`, BLOCK refuse)
4. Detect Agent (--agent or auto-detect)
5. Inject (copy files + SHA256 + manifest)
6. Launch (CLI: subprocess, IDE: clipboard + wait)
7. Wait (with Ctrl+C and --timeout support)
8. Cleanup (remove injected files, archive session)

## Build & Test

```bash
cargo build --workspace          # Build all
cargo test --workspace           # Run all tests (149+)
cargo build --release            # Release build
cargo run -- run ./skill "msg"   # Run CLI
cargo run -- scan ./skill        # Scan skill
cargo run -- agents              # List agents
cargo run -- info ./skill        # Show info
cargo run -- cache ls            # List cache
```

## Conventions

- Code, comments, docs, commits in English
- Frequent atomic commits
- Test fixtures in `cli/tests/fixtures/`
- v0.1 scanner uses regex (not tree-sitter)
- Agent adapters implement `AgentAdapter` trait with `async_trait` (11 total)
- Source fetchers use FetchContext structs to avoid excessive arguments
- Regex patterns in rules.rs must use `r#"..."#` (Rust 2021 raw strings)
- All user-facing output goes to stderr (via `eprintln!` / `ui::*`)
- JSON output goes to stdout (for piping)

## Data Directories

```
~/.skillx/
├── config.toml    # Global config
├── cache/         # Cached skills (TTL-based)
├── active/        # Active run sessions
└── history/       # Archived session manifests
```

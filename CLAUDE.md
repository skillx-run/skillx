# skillx Project Guide

> Run any agent skill. Safely. Without installing it.

**Core value props (in priority order):**
1. **No install needed** — `skillx run` is ephemeral by default: fetch, use, auto-clean. Nothing permanently added to the project.
2. **Security first** — 23 rules scan every skill before injection. Dangerous patterns are blocked.
3. **One command** — Full lifecycle (fetch → scan → inject → run → clean) in a single CLI call.

`skillx install` exists for persistent use cases but is opt-in, not the default.

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
  - `resolver.rs` — Unified resolve + fetch + cache abstraction (requires `&Config` param). `FetchedSkill` carries `resolved_ref` from source for version tracking
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
  - `markdown_analyzer.rs` — MD-001~006 (prompt injection, sensitive dirs, etc.) + MD-007 (license), MD-008 (name), MD-009 (description) structural checks
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
  - `inject.rs` — `inject_and_collect()` (core) + `inject_skill()` (manifest wrapper) + `InjectedRecord`/`InjectionType` + aggregate file ops
  - Signal handling via `tokio::signal::ctrl_c()` + `tokio::select!`
  - Interactive orphaned session recovery with metadata display
- `gate.rs` — Scan result gating (PASS/INFO auto-pass, WARN prompt, DANGER interactive, BLOCK refuse)
- `installed.rs` — Persistent install state (`~/.skillx/installed.json`)
- `cache.rs` — Cache management (SHA256 source hash, TTL)
- `config.rs` — `~/.skillx/config.toml` handling (incl. `[[url_patterns]]`, `[[custom_agents]]`)
- `project_config.rs` — `skillx.toml` project-level configuration ([skills] table format, `update_skill_source()` for version sync)
- `types.rs` — Shared types (Scope enum)
- `error.rs` — SkillxError (thiserror) + Result alias
- `ui.rs` — Terminal output helpers (console + indicatif)
- `commands/` — Command implementations (10 commands, anyhow::Result)
  - `run.rs` — Ephemeral run (fetch → scan → inject → launch → cleanup, the primary usage mode)
  - `install.rs` — Persistent install (opt-in, explicit sources or from skillx.toml)
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
cargo test --workspace           # Run all tests (300+)
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
- FetchedSkill carries resolved_ref from source for version tracking in installed state
- Agent version detection: `detect_binary_version()` runs `<binary> --version` and parses semver; `extract_vscode_extension_version()` parses from dir name. Both gracefully degrade to `None` on failure.
- SkillMetadata includes `license: Option<String>` field (parsed from frontmatter)
- MD-007 scanner rule: INFO level, triggers when frontmatter exists but has no `license` field (structural check in markdown_analyzer, not regex)
- MD-008/MD-009 scanner rules: INFO level, check for missing `name`/`description` fields in frontmatter (same structural pattern as MD-007)
- `installed.json` uses `scan_level: String` (intentional deviation from design doc's `scan_result` object — session manifest already stores full ScanReport for audit)
- Source fetchers distinguish HTTP 401/403/404 with platform-specific token guidance (GITLAB_TOKEN, BITBUCKET_TOKEN, etc.)
- `gate.rs` detail view shows file metadata (size, SHA-256, type) for binary/resource findings without line numbers
- Cleanup asks `[y/N]` before removing files modified during a session (SHA-256 mismatch detection)
- CI: GitHub Actions with `ci.yml` (fmt + clippy + test multi-platform + cargo-deny audit) and `release.yml` (tag → cross-compile → GitHub Release → crates.io → Homebrew tap)
- `deny.toml` configures cargo-deny for license allow-list and advisory checks
- Scanner WARN rules skip comment lines (script) and code blocks (markdown) to reduce false positives
- `install.sh` verifies SHA256 checksums before extraction (graceful degradation if unavailable)
- Homebrew formula template in `Formula/skillx.rb` (SHA256 placeholders replaced by release CI)
- cargo-binstall supported via `[package.metadata.binstall]` in Cargo.toml
- `install.sh` — Shell one-liner installer (`curl -fsSL https://skillx.run/install.sh | sh`)
- Web docs sidebar in `astro.config.mjs` lists all 10 commands (CLI Reference section lists `cache` not `config`; config.toml docs are in Reference section)
- `SKILLX_HOME` env var overrides the default `~/.skillx/` base directory (used by integration tests for isolation)
- GitHub Action at `.github/actions/scan/action.yml` — composite action for CI security scanning with SARIF upload
- `install` and `update` commands fetch skills concurrently (scan/gate remain sequential for interactive confirmation)
- `--print` / `-p` flag on `skillx run` enables non-interactive mode (agent processes prompt and exits)
- `LaunchConfig.print_mode` controls interactive vs non-interactive agent launch
- Agent prompt passing: Claude (`claude "msg"` / `claude -p "msg"`), Codex (`codex "msg"` / `codex exec "msg"`), Gemini (`gemini -i "msg"` / `gemini -p "msg"`), Amp (`amp -x "msg"`), OpenCode (`opencode "msg"` / `opencode run "msg"`)
- `skill_invocation_prefix()` trait method: default `/skill-name` (Agent Skills standard), Codex overrides to `$skill-name`, Goose/Aider return `None`
- `run` command auto-prepends skill invocation prefix to user prompt (e.g., `"/name-poem 李白"`); skips if user prompt already starts with prefix; generates prefix-only prompt when no user prompt given
- Agent YOLO flags: Claude (`--dangerously-skip-permissions`), Codex (`--yolo`), Gemini (`--yolo`), Amp (`--dangerously-allow-all`)
- `AgentDef` has `PromptStyle` (Flag/Positional/None), `PrintStyle` (Flag/Subcommand), `extra_launch_args`, `print_extra_args`, `aggregate_file`
- `PromptStyle`/`PrintStyle` chain setters: `.with_prompt_style()`, `.with_print_style()`, `.with_yolo()`, `.with_extra_args()`, `.with_aggregate_file()`
- `prepare_injection()` trait method on `AgentAdapter`: default raw-copy, GenericAdapter overrides for `aggregate_file` (Goose → `.goosehints`)
- `InjectedRecord` has `InjectionType` (CopiedFile/AggregateSection) for cleanup dispatch
- Aggregate file injection uses `<!-- skillx:begin:name -->` / `<!-- skillx:end:name -->` marker comments
- Amp injects to `.agents/skills/` (not `.amp/skills/`) — Amp reads `.agents/skills/` and `.claude/skills/`
- Aider: GenericAdapter auto-adds `--read SKILL.md` in launch when skill_dir has SKILL.md
- Most agents now natively support SKILL.md in `.<agent>/skills/` directories (Agent Skills standard)
- Example skills in `examples/skills/` (name-poem, hello-world, code-review, testing-guide, commit-message, dangerous-example)
- Web docs sidebar includes "Examples" section between Guides and Reference
- Web site is light-theme only (no dark mode) — ThemeSelect is overridden with empty component in astro.config.mjs

## Data Directories

```
~/.skillx/       (or $SKILLX_HOME if set)
├── config.toml      # Global config (url_patterns, custom_agents, cache, scan, agent, history)
├── installed.json   # Persistent install state (skills, injections, SHA256)
├── cache/           # Cached skills (TTL-based)
├── active/          # Active run sessions
└── history/         # Archived session manifests
```

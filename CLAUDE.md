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
- `source/` — Skill fetching (Local, GitHub)
- `scanner/` — Security scanning (MD/SC/RS rules)
- `agent/` — Agent detection & adapters (AgentAdapter trait)
- `session/` — Session lifecycle, manifest, inject, cleanup
- `cache.rs` — Cache management
- `config.rs` — `~/.skillx/config.toml` handling
- `types.rs` — Shared types (Scope enum)
- `error.rs` — SkillxError (thiserror) + Result alias
- `ui.rs` — Terminal output helpers (console + indicatif)
- `commands/` — Command implementations (anyhow::Result)

### Error Strategy

- `thiserror` (`SkillxError`): library modules
- `anyhow`: command layer (`commands/*.rs`)
- `main.rs`: catches anyhow::Error, formats via `ui::error()`

## Build & Test

```bash
cargo build --workspace          # Build all
cargo test --workspace           # Run all tests
cargo build --release            # Release build
cargo run -- run ./skill "msg"   # Run CLI
```

## Conventions

- Code, comments, docs, commits in English
- Frequent atomic commits
- Test fixtures in `cli/tests/fixtures/`
- v0.1 scanner uses regex (not tree-sitter)
- Agent adapters implement `AgentAdapter` trait with `async_trait`

## Dependencies

See `cli/Cargo.toml` for the full list. Key ones:
clap (CLI), tokio (async), reqwest (HTTP), serde (serialization),
thiserror/anyhow (errors), console/indicatif (UI), regex (scanning).

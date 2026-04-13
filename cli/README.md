# skillx CLI — Developer Guide

## Project Structure

```
cli/
├── src/
│   ├── main.rs                  # Entry point (thin clap shell)
│   ├── lib.rs                   # Re-exports all modules
│   ├── commands/                # Command implementations
│   │   ├── run.rs               # skillx run (full lifecycle)
│   │   ├── scan.rs              # skillx scan
│   │   ├── agents.rs            # skillx agents
│   │   ├── info.rs              # skillx info
│   │   └── cache.rs             # skillx cache ls|clean
│   ├── source/                  # Skill fetching
│   │   ├── mod.rs               # SkillSource enum, resolve(), parse_frontmatter()
│   │   ├── local.rs             # LocalSource::fetch()
│   │   └── github.rs            # GitHubSource::parse/parse_url/fetch()
│   ├── scanner/                 # Security scanning
│   │   ├── mod.rs               # ScanEngine, RiskLevel, Finding, ScanReport
│   │   ├── rules.rs             # All regex pattern constants
│   │   ├── markdown_analyzer.rs # MD-001~006
│   │   ├── script_analyzer.rs   # SC-001~011
│   │   ├── resource_analyzer.rs # RS-001~003
│   │   ├── binary_analyzer.rs   # Binary file metadata
│   │   └── report.rs            # TextFormatter, JsonFormatter
│   ├── agent/                   # Agent detection & adapters
│   │   ├── mod.rs               # AgentAdapter trait, types
│   │   ├── registry.rs          # AgentRegistry
│   │   ├── claude_code.rs       # Claude Code adapter
│   │   ├── codex.rs             # OpenAI Codex adapter
│   │   ├── copilot.rs           # GitHub Copilot adapter
│   │   ├── cursor.rs            # Cursor adapter
│   │   └── universal.rs         # Universal fallback
│   ├── session/                 # Session management
│   │   ├── mod.rs               # Session struct
│   │   ├── manifest.rs          # Manifest serialization
│   │   ├── inject.rs            # File injection with SHA256
│   │   └── cleanup.rs           # Cleanup + orphan recovery
│   ├── cache.rs                 # CacheManager
│   ├── config.rs                # Config (config.toml)
│   ├── error.rs                 # SkillxError enum
│   ├── types.rs                 # Scope enum
│   └── ui.rs                    # Terminal output helpers
├── tests/
│   ├── cli_parse_test.rs        # Clap argument parsing tests
│   ├── unit_tests.rs            # Comprehensive unit tests
│   └── fixtures/                # Test fixtures
│       ├── valid-skill/         # Clean skill
│       ├── dangerous-skill/     # Skill with security issues
│       ├── minimal-skill/       # Bare minimum skill
│       ├── binary-skill/        # Skill with ELF binary
│       └── no-skillmd/          # Directory without SKILL.md
└── Cargo.toml
```

## Development

```bash
# Build
cargo build

# Run tests
cargo test

# Run with arguments
cargo run -- run ./tests/fixtures/valid-skill "test prompt"
cargo run -- scan ./tests/fixtures/dangerous-skill
cargo run -- agents
cargo run -- info ./tests/fixtures/valid-skill
cargo run -- cache ls

# Check formatting
cargo fmt -- --check

# Lint
cargo clippy
```

## Adding a New Agent Adapter

1. Create `cli/src/agent/my_agent.rs`
2. Implement `AgentAdapter` trait (with `#[async_trait]`)
3. Add `pub mod my_agent;` to `cli/src/agent/mod.rs`
4. Register in `AgentRegistry::new()` in `cli/src/agent/registry.rs`
5. Add tests for inject paths and auto-approve args

## Adding a New Scanner Rule

1. Add pattern constant in `cli/src/scanner/rules.rs` (use `r#"..."#` format)
2. Add to the appropriate analyzer (markdown/script/resource)
3. Add test cases in `cli/tests/unit_tests.rs`
4. Update test fixtures if needed

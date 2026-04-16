# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.8.0] - 2026-04-17

### Added

- 7 new scanner rules bringing total to 30: SC-012 (base64/hex decode), SC-013 (string concatenation obfuscation), SC-014 (Unicode escape abuse), SC-015 (env var exfiltration), MD-010 (hidden zero-width characters), MD-011 (data/JavaScript URI), RS-005 (scripts in references directory)
- RS-004 symlink detection: scanner identifies and reports symlinks without following them
- Anti-evasion normalization layer: shell continuation-line joining and keyword whitespace normalization to detect obfuscated patterns
- Shebang detection for extensionless root files (scanned as scripts when `#!` is present)
- `--headless` flag and `CI=true` / `SKILLX_HEADLESS=1` env vars to disable interactive gate prompts
- `--fail-on` flag on `run` command to enforce minimum scan level threshold

### Fixed

- Fix tarball extraction failing on archives containing `pax_global_header` entries
- Fix symlink traversal security issue in scanner directory walking

### Changed

- Refactor gate API with `GateOptions` struct for cleaner headless and auto-approve control
- Optimize scanner performance: cache executable checks, efficient magic-byte reads for shebang detection, pre-compute normalization per line

## [0.7.0] - 2026-04-16

### Added

- `skillx upgrade` command: detect install method (Homebrew/Cargo/cargo-binstall) and execute upgrade, with manual instructions for unknown methods
- Background CLI version check after every command with two-tier fallback (GitHub Releases API → crates.io), rate-limited 24h cache, and configurable via `[update]` in config.toml
- `SKILLX_NO_UPDATE_CHECK` environment variable to disable background version check (useful in CI)

### Fixed

- Move `install.sh` to `web/public/` so it is correctly served at `skillx.run/install.sh`

## [0.6.0] - 2026-04-15

### Changed

- Refactor all source fetchers with three-tier download strategy: archive tarball → git clone → API fallback, avoiding API rate limits and improving download reliability across GitHub, GitLab, Bitbucket, Gitea, and SourceHut
- Adapt fetch tier order based on subpath presence: subpath requests use git sparse clone first (only downloads needed subdirectory), whole-repo requests use tarball first

### Fixed

- Fix skill caching not working: `CacheManager::write_meta()` now properly persists cache metadata after downloads
- Fix temp directory leaks on fetch failure paths with RAII cleanup guard
- Restore SourceHut platform-specific error messages (e.g., "Set SRHT_TOKEN") for 401/403/404 responses

## [0.5.0] - 2026-04-14

### Changed

- Rename `--yolo` CLI flag to `--auto-approve` / `--auto` for better clarity and alignment with "Security first" positioning
- Redesign website with neo-brutalist light-only theme
- Add release process documentation to CLAUDE.md

### Fixed

- Fix incorrect repo name in example paths (`skillx.run` → `skillx`)
- Fix terminal typing animation cursor width and steps

### Added

- Uninstall instructions for Shell Script and Homebrew installation methods
- X (@SkillxRun) social media link to website

## [0.4.0] - 2026-03-24

### Added

- `--print` / `-p` flag on `run` command for non-interactive mode (agent processes prompt and exits)
- Skill invocation prefix: `run` command auto-prepends `/skill-name` to user prompt (Agent Skills standard)
- `skill_invocation_prefix()` trait method with per-agent customization (Codex: `$skill-name`, Goose/Aider: `None`)
- `prepare_injection()` trait method for custom injection strategies (e.g., Goose → `.goosehints` aggregate file)
- Aggregate file injection with `<!-- skillx:begin:name -->` / `<!-- skillx:end:name -->` marker comments
- `PromptStyle` (Flag/Positional/None) and `PrintStyle` (Flag/Subcommand) in `AgentDef` for flexible agent command construction
- Agent auto-approve flags: Claude (`--dangerously-skip-permissions`), Codex (`--yolo`), Gemini (`--yolo`), Amp (`--dangerously-allow-all`)
- 5 example skills: name-poem, hello-world, code-review, testing-guide, commit-message (`examples/skills/`)
- Examples documentation section in web docs sidebar
- Landing page redesign: terminal hero with glow effect, install tabs, reordered sections
- PR preview deployments via Cloudflare Pages
- README badges (CI, crates.io, downloads, license, docs)

### Fixed

- Agent injection paths (Amp injects to `.agents/skills/`, not `.amp/skills/`)
- Aggregate path joining bug for nested injection directories
- Clippy warnings in `inject.rs`
- `cargo-deny` advisory IDs (RUSTSEC-2025-0057, RUSTSEC-2025-0119)
- Web: numerous docs accuracy fixes (scanner rules count, platform support status, agent list, config.toml sections)

### Changed

- Aider adapter auto-adds `--read SKILL.md` in launch when skill directory contains SKILL.md
- Web: complete documentation overhaul (installation, run command, platforms, agents, scanner, config.toml, CI integration)
- Web: ecosystem section redesigned with card layout and visual hierarchy

## [0.3.2] - 2026-03-22

### Added

- `SKILLX_HOME` environment variable to override the default `~/.skillx/` data directory (useful for testing and isolated environments)
- GitHub Action for CI security scanning (`.github/actions/scan/action.yml`) with SARIF upload to GitHub Code Scanning
- Example scan workflow (`.github/workflows/scan-example.yml`)
- `cache` command documentation page (`web/src/content/docs/cli/cache.md`)
- Command integration tests for install, uninstall, list, and update (15 new tests using `SKILLX_HOME` isolation)

### Changed

- `install` and `update` commands now fetch multiple skills concurrently, then scan/gate sequentially (improves performance for multi-skill operations)
- `install` reports partial failures when some sources fail to fetch instead of aborting entirely
- CI integration guide updated with official GitHub Action usage and SARIF upload examples
- Web docs sidebar: replaced `config` with `cache` in CLI Reference section (config.toml docs remain in Reference section)

## [0.3.1] - 2026-03-22

### Added

- MD-008 scanner rule: INFO level when frontmatter exists but has no `name` field
- MD-009 scanner rule: INFO level when frontmatter exists but has no `description` field
- Binary finding detail view shows file metadata (size, SHA-256, detected type)
- Shell one-liner install script (`install.sh`) for Linux and macOS with SHA256 checksum verification
- E2E integration tests for scan, agents, info, init, cache, and CLI basics
- `cargo-deny` CI job for security and license auditing (`deny.toml`)
- Homebrew formula (`brew install skillx-run/tap/skillx`) with auto-update on release
- Scanner edge case tests (empty files, long lines, comment/code-block awareness)

### Fixed

- Replace 6 `unwrap()` calls in production code with proper error propagation
- Scanner MD-003 no longer fires on plain URLs (removed overly broad `https?://\S+` pattern)
- Scanner WARN rules (SC-006/007/008) no longer fire on comment lines (`#`, `//`, `--`)
- Scanner WARN rules (MD-003/004) no longer fire inside fenced code blocks
- SC-009 regex now correctly requires 4-digit octal permissions (was matching 3-digit modes like `644`)
- `history.max_entries` config setting now respected (was hardcoded to 50)
- `NoAgentDetected` error now includes actionable guidance (install agent or use --agent)
- HTTP 401/403/404 responses from GitLab, Bitbucket, Gitea, Gist, SourceHut, and HuggingFace now show specific error messages with token environment variable guidance

### Changed

- Unified GitHub organization from `anthropics/skillx` to `skillx-run/skillx` across all project references

## [0.3.0] - 2026-03-21

### Added

- 10 source types: Local, GitHub, GitLab, Bitbucket, Gitea, Gist, SourceHut, HuggingFace, Archive, SkillsDirectory
- 32 agent adapters across three tiers plus custom and universal fallback
  - Tier 1: Claude Code, Codex, Copilot, Cursor
  - Tier 2: Gemini CLI, OpenCode, Amp, Windsurf, Cline, Roo
  - Tier 3: 21 agents via data-driven generic adapter
  - User-defined custom agents from `config.toml` `[[custom_agents]]`
- URL smart recognition engine supporting 20+ platforms
- `config.toml` support for custom `[[url_patterns]]` and `[[custom_agents]]`
- `skillx.toml` `[skills]` table format with string shorthand and detailed object syntax
- `install --prune`, `--dev`, `--prod`, `--no-save` flags
- `list --outdated` with dedicated table format
- `update --dry-run` with confirmation prompt and per-skill progress saving
- `license` field to `SkillMetadata` (parsed from frontmatter)
- MD-007 scanner rule: INFO level when frontmatter exists but has no `license` field
- `install --all` to install to all detected agents simultaneously
- Conflict detection: active session check, upgrade detection, unmanaged path prompt

### Changed

- `install` now saves sources to skillx.toml by default (use `--no-save` to skip)
- Cleanup prompts `[y/N]` before removing user-modified files (TTY only)

## [0.2.0] - 2026-03-19

### Added

- Persistent skill management commands: `install`, `uninstall`, `list`, `update`, `init`
- `installed.json` state tracking with SHA-256 content hashing
- Multi-agent injection support (install skills to multiple agents)
- `skillx.toml` project configuration for declaring project skill dependencies
- `--attach` flag for attaching context files to sessions
- `--stdin` and `--prompt-file` input modes
- Session orphan recovery with metadata display

## [0.1.0] - 2026-03-17

### Added

- Initial release of skillx
- Core commands: `run`, `scan`, `agents`, `info`, `cache`
- Security scanner with 20 rules across three analyzers
  - Markdown: MD-001 through MD-006
  - Script: SC-001 through SC-011
  - Resource: RS-001 through RS-003
- 5 risk levels: Pass, Info, Warn, Danger, Block
- Tier 1 agent support: Claude Code, Codex, Copilot, Cursor
- GitHub source support with TTL-based caching
- Session lifecycle management with Ctrl+C signal handling
- SARIF 2.1.0 output format for scanner results
- Text and JSON output formatters

[0.8.0]: https://github.com/skillx-run/skillx/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/skillx-run/skillx/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/skillx-run/skillx/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/skillx-run/skillx/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/skillx-run/skillx/compare/v0.3.2...v0.4.0
[0.3.2]: https://github.com/skillx-run/skillx/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/skillx-run/skillx/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/skillx-run/skillx/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/skillx-run/skillx/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/skillx-run/skillx/releases/tag/v0.1.0

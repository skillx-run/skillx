# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- MD-008 scanner rule: INFO level when frontmatter exists but has no `name` field
- MD-009 scanner rule: INFO level when frontmatter exists but has no `description` field
- Binary finding detail view shows file metadata (size, SHA-256, detected type)
- Shell one-liner install script (`install.sh`) for Linux and macOS
- E2E integration tests for scan, agents, info, init, cache, and CLI basics

### Fixed

- `history.max_entries` config setting now respected (was hardcoded to 50)
- `NoAgentDetected` error now includes actionable guidance (install agent or use --agent)
- HTTP 401/403/404 responses from GitLab, Bitbucket, Gitea, Gist, SourceHut, and HuggingFace now show specific error messages with token environment variable guidance

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

[0.3.0]: https://github.com/skillx-run/skillx/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/skillx-run/skillx/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/skillx-run/skillx/releases/tag/v0.1.0

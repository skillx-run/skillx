# Contributing to skillx

Thank you for your interest in contributing to skillx! This guide will help you get started.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Reporting Bugs](#reporting-bugs)
- [Suggesting Features](#suggesting-features)
- [Development Setup](#development-setup)
- [Project Structure](#project-structure)
- [Code Style](#code-style)
- [Commit Conventions](#commit-conventions)
- [Pull Request Process](#pull-request-process)

## Code of Conduct

This project follows the [Contributor Covenant Code of Conduct](https://www.contributor-covenant.org/version/2/1/code_of_conduct/). By participating, you are expected to uphold this code. Please report unacceptable behavior via [GitHub Issues](https://github.com/anthropics/skillx/issues).

## Reporting Bugs

Open a [GitHub Issue](https://github.com/anthropics/skillx/issues/new) with:

- A clear, descriptive title
- Steps to reproduce the problem
- Expected vs. actual behavior
- Your environment: OS, Rust version (`rustc --version`), skillx version (`skillx --version`)
- Relevant logs or error output

**Security vulnerabilities** should NOT be reported via public issues. See [SECURITY.md](SECURITY.md) instead.

## Suggesting Features

Open a [GitHub Issue](https://github.com/anthropics/skillx/issues/new) labeled `enhancement` with:

- A description of the problem the feature would solve
- Your proposed solution or approach
- Any alternatives you have considered

## Development Setup

### Prerequisites

- [Rust toolchain](https://rustup.rs/) (stable, latest)
- Git

### Build and Test

```bash
# Clone the repository
git clone https://github.com/anthropics/skillx.git
cd skillx

# Build all workspace members
cargo build --workspace

# Run all tests
cargo test --workspace

# Run the CLI locally
cargo run -- run ./skill "message"
cargo run -- scan ./skill
cargo run -- agents --all
```

## Project Structure

This is a monorepo with three components:

```
skillx/
├── cli/        # Rust CLI tool (the core project)
├── web/        # Astro + Starlight documentation site
└── registry/   # Cloudflare Workers API (placeholder)
```

Most contributions will target `cli/`. Key directories within it:

- `cli/src/commands/` — Command implementations
- `cli/src/source/` — Skill fetching from multiple platforms
- `cli/src/scanner/` — Security scanning engine
- `cli/src/agent/` — Agent detection and adapters
- `cli/src/session/` — Session lifecycle management
- `cli/tests/` — Integration tests
- `cli/tests/fixtures/` — Test fixtures

## Code Style

- Run `cargo fmt` before committing — all code must be formatted
- Run `cargo clippy` and resolve all warnings
- All user-facing output goes to stderr (via `eprintln!` / `ui::*`)
- JSON output goes to stdout (for piping)
- Use `thiserror` for library error types, `anyhow` for command-layer errors
- Regex patterns must use raw string syntax: `r#"..."#`

## Commit Conventions

- **Language**: All code, comments, documentation, and commit messages must be in English
- **Style**: Use semantic commit messages:
  - `feat:` — New feature
  - `fix:` — Bug fix
  - `docs:` — Documentation changes
  - `test:` — Adding or updating tests
  - `refactor:` — Code change that neither fixes a bug nor adds a feature
  - `chore:` — Build process, dependency updates, etc.
- **Scope**: Keep commits atomic — one small, self-contained change per commit
- **Examples**:
  - `feat: add GitLab source support`
  - `fix: handle empty manifest on cleanup`
  - `test: add scanner edge case for nested archives`

## Pull Request Process

1. **Fork** the repository and create a branch from `main`
2. **Make your changes** following the code style and commit conventions above
3. **Add tests** for any new functionality — include normal paths, edge cases, and error handling
4. **Ensure all checks pass**:
   ```bash
   cargo fmt --check
   cargo clippy --workspace
   cargo test --workspace
   ```
5. **Open a PR** against `main` with:
   - A clear title and description of the change
   - Reference to any related issues (e.g., `Fixes #123`)
   - A summary of what was changed and why
6. **Address review feedback** — maintainers may request changes before merging

### What We Look For

- Tests covering new and changed behavior
- No new clippy warnings
- Consistent error handling following the project's `thiserror`/`anyhow` split
- Documentation updates if user-facing behavior changes

## Contributor License Agreement

By submitting a pull request, you represent that you have the right to license your contribution to the project, and you agree that your contribution will be licensed under the project's existing licenses:

- Code: [Apache License 2.0](LICENSE)
- Documentation: [CC-BY-4.0](LICENSE-DOCS)

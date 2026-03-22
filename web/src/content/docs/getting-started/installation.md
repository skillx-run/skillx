---
title: Installation
description: How to install skillx via shell script, Homebrew, Cargo, cargo-binstall, or from source.
---

## Requirements

- **Operating System**: macOS, Linux, or Windows
- **Agent**: At least one supported agent installed (see `skillx agents --all` for the full list of 32+ supported agents)

## Install via Shell Script (Recommended)

The fastest way to install skillx on macOS or Linux:

```bash
curl -fsSL https://skillx.run/install.sh | sh
```

The installer automatically detects your OS and architecture, downloads a pre-compiled binary, verifies SHA256 checksums, and installs to `~/.local/bin/`.

## Install via Homebrew (macOS / Linux)

```bash
brew install skillx-run/tap/skillx
```

To update:

```bash
brew upgrade skillx
```

## Install via Cargo

If you have the Rust toolchain installed:

```bash
cargo install skillx
```

This builds from source and places the `skillx` binary in `~/.cargo/bin/`.

To update to the latest version:

```bash
cargo install skillx --force
```

### Installing Rust

If you don't have Rust yet:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Then restart your shell or run `source ~/.cargo/env`.

## Install via cargo-binstall (fast binary install)

If you have [cargo-binstall](https://github.com/cargo-bins/cargo-binstall) installed, you can download a pre-compiled binary instead of building from source:

```bash
cargo binstall skillx
```

## Build from Source

Clone the repository and build:

```bash
git clone https://github.com/skillx-run/skillx.git
cd skillx
cargo build --release
```

The binary will be at `target/release/skillx`. Move it to a directory in your `$PATH`:

```bash
cp target/release/skillx /usr/local/bin/
```

### Development Build

For development with debug symbols:

```bash
cargo build
# Binary at target/debug/skillx

# Run directly without installing
cargo run -- run ./my-skill "prompt"
cargo run -- scan ./my-skill
```

## Verify Installation

```bash
skillx --version
```

You should see output like:

```
skillx 0.3.2
```

Check that an agent is detected:

```bash
skillx agents
```

Example output:

```
Agent Environments

  Claude Code [✓ detected]
    claude binary found
    Lifecycle: ManagedProcess
    YOLO: --dangerously-skip-permissions
```

## Data Directories

On first run, skillx creates its configuration directory:

```
~/.skillx/
├── config.toml    # Global configuration (optional)
├── cache/         # Cached skills (TTL-based)
├── active/        # Active run sessions
└── history/       # Archived session manifests
```

You can customize behavior by creating `~/.skillx/config.toml`. See [config.toml reference](/reference/config-toml/) for details.

## Uninstall

### Cargo

```bash
cargo uninstall skillx
rm -rf ~/.skillx  # Remove data directory
```

## Troubleshooting

### "command not found: skillx"

Ensure `~/.cargo/bin` is in your `$PATH`. Add this to your shell config:

```bash
# ~/.bashrc or ~/.zshrc
export PATH="$HOME/.cargo/bin:$PATH"
```

### "No agents detected"

skillx needs at least one supported agent. Install one:

- **Claude Code**: `npm install -g @anthropic-ai/claude-code`
- **Codex**: See [OpenAI Codex docs](https://github.com/openai/codex)
- **Copilot**: Install the GitHub Copilot extension in VS Code
- **Cursor**: Download from [cursor.com](https://cursor.com)

If no specific agent is detected, skillx falls back to the universal adapter which injects files into `.agents/skills/`.

### Build errors

Ensure you have a C compiler and OpenSSL development headers:

```bash
# Ubuntu/Debian
sudo apt install build-essential pkg-config libssl-dev

# macOS (install Xcode command line tools)
xcode-select --install

# Fedora
sudo dnf install gcc openssl-devel
```

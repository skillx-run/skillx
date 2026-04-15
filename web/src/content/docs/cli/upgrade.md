---
title: "skillx upgrade"
description: Reference for the skillx upgrade command — check for and upgrade the skillx CLI itself.
---

## Synopsis

```bash
skillx upgrade
```

Check if a newer version of skillx is available and upgrade the CLI tool itself.

## How It Works

### 1. Version Check

Queries the latest published version and compares it with the currently running binary using semantic versioning.

Two sources are checked in order:

1. **GitHub Releases API** — primary, most authoritative (tracks release tags). Uses `GITHUB_TOKEN` if available for higher rate limits.
2. **crates.io API** — fallback, independent infrastructure. Used automatically if the GitHub check fails (e.g., rate limiting).

### 2. Install Method Detection

skillx detects how it was installed by examining the binary's filesystem path:

| Path pattern | Detected method |
|--------------|-----------------|
| `/opt/homebrew/Cellar/...` | Homebrew |
| `~/.cargo/bin/skillx` | Cargo (or cargo-binstall if available) |
| Other | Unknown |

### 3. Upgrade Execution

Based on the detected install method:

| Method | Action |
|--------|--------|
| **Homebrew** | Runs `brew upgrade skillx` |
| **cargo-binstall** | Runs `cargo binstall skillx -y` (fast, pre-built binary) |
| **Cargo** | Runs `cargo install skillx` (compiles from source) |
| **Unknown** | Shows manual upgrade instructions |

When the install method cannot be determined, skillx displays the available installation options instead of attempting an automatic upgrade.

## Background Update Check

In addition to the explicit `upgrade` command, skillx performs a non-blocking version check after every command (except `upgrade` itself). If a newer version is detected, a notification is printed:

```
ℹ A new version of skillx is available: v0.6.0 → v0.7.0
  Run `skillx upgrade` to update.
```

This check:
- Uses a cached result to avoid excessive API calls (default interval: 24 hours)
- Never blocks or slows down the running command
- Silently ignores network errors
- Can be disabled (see below)

## Configuration

Control the background update check via `~/.skillx/config.toml`:

```toml
[update]
check = true    # Set to false to disable background checks
interval = "24h" # How often to query GitHub (e.g., "1h", "7d")
```

Or disable it entirely with an environment variable:

```bash
export SKILLX_NO_UPDATE_CHECK=1
```

## Examples

### Check and upgrade

```bash
skillx upgrade
```

### Disable background checks in CI

```bash
SKILLX_NO_UPDATE_CHECK=1 skillx run ./my-skill
```

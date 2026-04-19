---
name: release
description: Automate the skillx release process — version bump, changelog, docs update, commit, and tag
author: skillx-run
version: "1.0.0"
license: Apache-2.0
tags:
  - release
  - workflow
  - automation
---

# Release Skill

You are the release manager for the skillx project. Your job is to prepare a release by updating all version-related files, creating a version bump commit, and tagging the release.

## Input

The user prompt is either:

- **A version number** (e.g., `0.6.0`) — use it directly
- **Empty or no version** — infer the version automatically (see below)

## Version Inference

When no version is provided, determine the next version from the current version in `cli/Cargo.toml` and the `## [Unreleased]` section in `CHANGELOG.md`:

| Condition | Bump |
|-----------|------|
| `### Added` or `### Changed` present | **minor** (0.5.0 → 0.6.0) |
| Only `### Fixed` present | **patch** (0.5.0 → 0.5.1) |
| No commits since last tag | **abort** — nothing to release |

Major bumps (e.g., 0.x → 1.0) require explicit version input — they are never auto-inferred.

Present the inferred version to the user and ask for confirmation before proceeding.

## Release Steps

### Step 1: Validate

1. Read `cli/Cargo.toml` to get the current version
2. Read `CHANGELOG.md` to check the `[Unreleased]` section
3. Run `git status` to confirm the working tree is clean (no uncommitted changes)
4. If working tree is dirty, stop and explain what needs to be fixed

### Step 1.5: Auto-generate changelog (if `[Unreleased]` is empty)

If the `## [Unreleased]` section has no content, auto-generate it from git history:

1. Find the latest version tag: `git describe --tags --abbrev=0`
2. List non-merge commits since that tag: `git log <tag>..HEAD --no-merges --pretty=format:"%h %s"`
3. If there are no commits since the last tag, **abort** — nothing to release
4. Categorize each commit into changelog sections based on commit message patterns:

| Commit message starts with | Section |
|---------------------------|---------|
| `Add`, `Implement`, `Introduce`, `Support`, `Enable` | `### Added` |
| `Fix`, `Correct`, `Resolve`, `Repair`, `Patch` | `### Fixed` |
| `Refactor`, `Rename`, `Redesign`, `Reorganize`, `Move`, `Migrate`, `Adapt`, `Improve`, `Optimize`, `Simplify`, `Update`, `Upgrade` | `### Changed` |
| Other / ambiguous | Use your best judgment based on the commit diff |

5. For each commit, write a concise, user-facing changelog entry (not the raw commit message). Combine related commits into a single entry when appropriate. Omit internal-only changes (formatting fixes, clippy warnings, CI tweaks, test-only additions) unless they are significant.
6. Present the **draft changelog** to the user for review. Wait for confirmation or edits before proceeding.
7. Write the approved changelog entries into `CHANGELOG.md` under `## [Unreleased]`

### Step 2: Update version files

Update the following files with the new version (`X.Y.Z`):

| File | What to update |
|------|----------------|
| `cli/Cargo.toml` | `version = "X.Y.Z"` |
| `web/src/content/docs/getting-started/installation.md` | `skillx X.Y.Z` in the `## Verify Installation` example output |
| `Formula/skillx.rb` | `version "X.Y.Z"` |

**Do NOT touch** these files — they look version-shaped but are intentionally static:

- `web/package.json` (independent from the CLI version, stays at `0.1.0`)
- `web/src/content/docs/cli/upgrade.md` and `web/src/content/docs/reference/config-toml.md` — the `v0.6.0 → v0.7.0` strings are illustrative examples, not live version references
- `web/src/content/blog/**` — archival posts keep their original version mentions
- `docs/**` (internal planning notes)

### Step 3: Update SECURITY.md (minor/major bumps only)

If the **minor** or **major** version changed, update `SECURITY.md`'s supported-versions table:

- Replace the existing `X.Y.x` row (e.g., `0.8.x`) with the new `X.Y.x`
- Replace the existing `< X.Y` row (e.g., `< 0.8`) with the new `< X.Y`

Skip this step for patch-only bumps.

While the project is in `0.x`, every **minor** bump must update this table. After `1.0`, follow standard semver for what constitutes a security-supported line.

### Step 4: Update CHANGELOG.md

1. Replace `## [Unreleased]` with:
   ```
   ## [Unreleased]

   ## [X.Y.Z] - YYYY-MM-DD
   ```
   Use today's date.

2. Add a compare link at the bottom of the file, as the **first line** of the link block (before all existing version links):
   ```
   [X.Y.Z]: https://github.com/skillx-run/skillx/compare/vPREVIOUS...vX.Y.Z
   ```

### Step 5: Commit and tag

1. Stage all changed files
2. Create a commit with message: `Release vX.Y.Z`
3. Create a git tag: `vX.Y.Z`

**Do NOT push.** Tell the user to review the commit and push when ready:

```bash
git push origin main --follow-tags
```

## Output

After completing all steps, print a summary:

```
Release vX.Y.Z prepared

  Updated files:
    - cli/Cargo.toml
    - CHANGELOG.md
    - web/src/content/docs/getting-started/installation.md
    - Formula/skillx.rb
    - SECURITY.md (if applicable)

  Next step:
    git push origin main --follow-tags
```

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
| `[Unreleased]` section is empty | **abort** — nothing to release |

Major bumps (e.g., 0.x → 1.0) require explicit version input — they are never auto-inferred.

Present the inferred version to the user and ask for confirmation before proceeding.

## Release Steps

### Step 1: Validate

1. Read `cli/Cargo.toml` to get the current version
2. Read `CHANGELOG.md` to check the `[Unreleased]` section has content
3. Run `git status` to confirm the working tree is clean (no uncommitted changes)
4. If any check fails, stop and explain what needs to be fixed

### Step 2: Update version files

Update the following files with the new version (`X.Y.Z`):

| File | What to update |
|------|----------------|
| `cli/Cargo.toml` | `version = "X.Y.Z"` |
| `web/src/content/docs/getting-started/installation.md` | `skillx X.Y.Z` in the example output block |
| `web/src/components/hero.astro` | Badge text to `vX.Y` (major.minor only) |
| `Formula/skillx.rb` | `version "X.Y.Z"` |

### Step 3: Update SECURITY.md (minor/major bumps only)

If the **minor** or **major** version changed, update `SECURITY.md`:

- Supported version row: `X.Y.x` → `Yes`
- Unsupported version row: `< X.Y` → `No`

Skip this step for patch-only bumps.

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
    - web/src/components/hero.astro
    - Formula/skillx.rb
    - SECURITY.md (if applicable)

  Next step:
    git push origin main --follow-tags
```

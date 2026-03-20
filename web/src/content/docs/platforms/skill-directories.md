---
title: Skill Directories
description: Curated skill registries and marketplaces that can be used with skillx.
---

## Overview

Skill directories are curated registries and marketplaces where skill authors publish their Agent Skills. While skillx v0.1 fetches skills directly from Git hosts, future versions will integrate with these directories for discovery, search, and one-click installation.

## Planned Integrations

### skills.sh

A community-driven, open directory of Agent Skills with a focus on quality and security.

```bash
# Planned syntax
skillx run skills.sh/pdf-processing "prompt"
```

Features:
- Community-curated skill listings
- Automated security scanning on publish
- Version pinning and changelogs
- Author verification

### skillsmp (Skills Marketplace)

A marketplace for both free and commercial skills.

```bash
# Planned syntax
skillx run skillsmp:author/skill-name "prompt"
```

Features:
- Free and paid skills
- Reviews and ratings
- Usage analytics for authors
- License management

### ClawHub

A hub focused on Claude Code skills and extensions.

```bash
# Planned syntax
skillx run clawhub:skill-name "prompt"
```

Features:
- Optimized for Claude Code workflows
- Skill composition (skills that depend on other skills)
- CLAUDE.md template integration

### LobehHub

Part of the Lobe ecosystem, focused on AI agent configurations and skills.

```bash
# Planned syntax
skillx run lobehub:agent-skill "prompt"
```

Features:
- Integration with LobeChat agent ecosystem
- Multi-agent skill orchestration
- Community-driven skill sharing

## Using Directory Skills Today

Until native directory integration ships, you can use skills from any directory that publishes to a Git host:

```bash
# If a skill directory publishes to GitHub
skillx run github:skills-sh/registry/pdf-processing "prompt"

# Or clone and use locally
git clone https://github.com/skills-sh/registry.git
skillx run ./registry/pdf-processing "prompt"
```

## How Discovery Will Work

The planned workflow for directory integration:

### 1. Search

```bash
# Planned command
skillx search "pdf processing"
```

```
Skills matching "pdf processing":

  skills.sh/pdf-extract     Extract tables and text from PDFs
  skills.sh/pdf-summarize   Summarize PDF documents
  skillsmp:acme/pdf-pro     Professional PDF processing suite
```

### 2. Inspect

```bash
# Planned command
skillx info skills.sh/pdf-extract
```

```
Name:        pdf-extract
Author:      @pdftools
Version:     1.2.0
Downloads:   12,450
Rating:      4.8/5
Scan Status: PASS
```

### 3. Run

```bash
skillx run skills.sh/pdf-extract "Extract tables from report.pdf"
```

## Registry API

The skillx registry API (planned for v0.4+) will provide:

| Endpoint | Description |
|----------|-------------|
| `GET /skills` | List skills with search and filtering |
| `GET /skills/:name` | Get skill metadata |
| `GET /skills/:name/versions` | List versions |
| `POST /skills` | Publish a skill (authenticated) |
| `GET /scan/:name` | Get latest scan report |

The registry will be implemented as a Cloudflare Workers API (see `registry/` in the monorepo).

## Publishing Skills

To make your skill discoverable:

1. **GitHub**: Push to a public repo with a `SKILL.md` at the root or in a subdirectory
2. **Directories**: Follow the directory's publishing guidelines (coming soon)

Ensure your skill passes `skillx scan --fail-on warn` before publishing:

```bash
skillx scan --fail-on warn ./my-skill
```

See [Writing Skills](/guides/writing-skills/) for the complete guide on creating publishable skills.

---
title: Skill Directories
description: Curated skill registries and marketplaces that can be used with skillx.
---

## Overview

Skill directories are curated registries and marketplaces where skill authors publish their Agent Skills. skillx v0.2 supports fetching skills from directory platforms by extracting the underlying GitHub source URL from the platform's page.

## Supported Platforms (10)

| Platform | Domain | Status |
|----------|--------|--------|
| skills.sh | `skills.sh` | Supported |
| Skills Marketplace | `skillsmp.com` | Supported (API) |
| ClawHub | `clawhub.ai` | Supported |
| LobeHub | `lobehub.com` | Supported |
| SkillHub | `skillhub.club` | Supported |
| Agent Skills Hub | `agentskillshub.dev` | Supported |
| Agent Skills | `agentskills.so` | Supported |
| MCP Market | `mcpmarket.com` | Supported |
| Skills Directory | `skillsdirectory.com` | Supported |
| Prompts Chat | `prompts.chat` | Supported |

### Usage

Simply pass the platform URL to skillx:

```bash
skillx run https://skills.sh/pdf-processing "prompt"
skillx run https://skillsmp.com/skills/code-review "prompt"
```

### How It Works

skillx extracts the underlying source repository (typically GitHub) from the platform:

1. **API first**: For platforms with APIs (e.g., skillsmp.com), queries the API for the source URL
2. **HTML parsing**: Falls back to parsing the page HTML, extracting GitHub links from `<a>` tags
3. **Meta tags**: Checks Open Graph and other meta tags for source repository links

Once the GitHub URL is extracted, fetching proceeds as normal through the GitHub source adapter.

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

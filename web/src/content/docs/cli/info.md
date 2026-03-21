---
title: "skillx info"
description: Reference for the skillx info command — display metadata and file listing for a skill.
---

## Synopsis

```bash
skillx info <source>
```

Display metadata and file listing for a skill without installing or running it. Useful for inspecting a skill before use.

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `source` | Yes | Skill source: local path, `github:`/`gist:` prefix, or URL |

## Output

skillx resolves and fetches the skill source (using cache if available), parses the SKILL.md frontmatter for metadata, and displays the following information:

```
Skill Info

  Name:         pdf-processing
  Description:  Extract and analyze PDF documents
  Author:       anthropics
  Version:      v1.2
  License:      MIT
  Tags:         pdf, extraction, analysis
  Source:       github:anthropics/skills/pdf-processing@v1.2
  Path:         ~/.skillx/cache/a1b2c3d4/pdf-processing

  Files:
    SKILL.md
    scripts/extract.py
    scripts/analyze.py
    references/schema.json
```

### Metadata Fields

| Field | Source |
|-------|--------|
| **Name** | SKILL.md frontmatter `name` field, or directory name |
| **Description** | SKILL.md frontmatter `description` field |
| **Author** | SKILL.md frontmatter `author` field |
| **Version** | Resolved ref from source (e.g., tag, branch, commit) |
| **License** | SKILL.md frontmatter `license` field |
| **Tags** | SKILL.md frontmatter `tags` field |
| **Source** | The resolved source string |
| **Path** | Local path to the skill (cache directory or local source) |

Fields that are not present in the SKILL.md frontmatter are shown with placeholder values such as `(unnamed)`, `(none)`, or `(unknown)`. The Tags field is only shown when tags are defined.

## Examples

### Inspect a local skill

```bash
skillx info ./my-skill
```

### Inspect a GitHub skill

```bash
skillx info github:anthropics/skills/pdf-processing
```

### Inspect a remote URL skill

```bash
skillx info https://github.com/org/repo/tree/main/skills/formatter
```

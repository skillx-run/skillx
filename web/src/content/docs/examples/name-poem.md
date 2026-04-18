---
title: "Name Poem"
description: A multilingual poetry skill that transforms names into classical Chinese acrostic poetry, haiku, sijo, sonnets, and more.
---

The `name-poem` skill transforms your AI agent into a multilingual poet. Give it any name, and it composes a poem in the poetic tradition that best honors that name's language and culture — classical Chinese acrostic poetry for Chinese names, haiku for Japanese names, sijo for Korean names, acrostic poems for English names, and more.

## Design Rationale

Names carry meaning across every culture, and every culture has its own poetic tradition. This skill bridges them:

- **Cultural awareness** — automatically selects the poetic form native to the name's language
- **Structured output** — name analysis, poem, translation, and commentary in a consistent format
- **Rich prompt engineering** — detailed composition guidelines for each tradition (meter, tonal balance, seasonal words)
- **Positive framing** — every name becomes a poem, regardless of length or origin

## SKILL.md

The skill is a single-file directory:

```
name-poem/
└── SKILL.md
```

Key sections of the SKILL.md:

- **Poetic Traditions** — mapping table from name origin to poetic form and technique
- **Composition Guidelines** — detailed rules for Chinese (meter, tonal balance), Japanese (mora, seasonal words), Korean (sijo structure), English (acrostic flow), and other traditions
- **Output Format** — name analysis, poem, translation, commentary
- **Example** — Li Bai rendered as a classical Chinese acrostic
- **Creative Spirit** — how to handle short names, compound names, and multicultural names

## Usage Scenarios

### Chinese Name — Classical Acrostic Poetry

```bash
skillx run github:skillx-run/skillx/examples/skills/name-poem "李白"
```

The agent composes a classical Chinese acrostic poem where each line begins with a character from the name, following 5-char or 7-char meter.

### Japanese Name — Haiku

```bash
skillx run github:skillx-run/skillx/examples/skills/name-poem "田中太郎"
```

A 5-7-5 mora haiku inspired by the kanji meanings, with a seasonal word woven in.

### English Name — Acrostic Poem

```bash
skillx run github:skillx-run/skillx/examples/skills/name-poem "Alice"
```

Each line begins with a successive letter: A-L-I-C-E, with vivid imagery and rhythmic flow.

### Korean Name — Sijo

```bash
skillx run github:skillx-run/skillx/examples/skills/name-poem "김민준"
```

A three-line sijo with the name's meaning blooming through natural imagery.

### Run from a Local Clone of this Repository

If you are already inside a local clone of `skillx-run/skillx`, use:

```bash
skillx run ./examples/skills/name-poem "Your Name"
```

## Scan Output

Scanning the name-poem skill:

```bash
skillx scan github:skillx-run/skillx/examples/skills/name-poem
```

```
  Scanning  name-poem
  ────────────────────────────────
  ✓ PASS — No issues found

  Files scanned: 1
  Risk level: PASS
```

The skill is purely instructional — no scripts, no URLs, no sensitive references — so it passes cleanly.

If you want to scan the local checkout instead, run:

```bash
skillx scan ./examples/skills/name-poem
```

## Team Configuration with skillx.toml

Add the name-poem skill to your project for creative team-building or onboarding:

```toml
[project]
name = "my-project"

[skills]
name-poem = "github:skillx-run/skillx/examples/skills/name-poem"
```

### Combining with Other Skills

```toml
[skills]
name-poem = "github:skillx-run/skillx/examples/skills/name-poem"
code-review = "github:skillx-run/skillx/examples/skills/code-review"
commit-message = "github:skillx-run/skillx/examples/skills/commit-message"
```

## Why this example exists

This example exists to show that a skill can be expressive without becoming vague: it still uses explicit rules for tradition, structure, and output, but it leaves room for a distinct creative result.

## Next Steps

If you want to see another example of structured output, compare it with [Code Review](/examples/code-review/). If you want to create a skill with a different voice or format, read [Writing Skills](/guides/writing-skills/). If you would rather try a polished external workflow than build from scratch, visit [Famous Skills](/getting-started/famous-skills/).

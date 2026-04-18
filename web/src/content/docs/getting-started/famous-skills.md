---
title: Famous Skills
description: Curated external GitHub skills you can run immediately with skillx.
---

Famous Skills are curated external GitHub skills from well-known repositories. They are ranked for immediate usefulness: start with the one that matches your current task, then move to the others when you need a narrower workflow.

If you only try one, start with **Frontend Design**. It shows the strongest end-to-end value for a first run: a concrete prompt produces visible output, and it is the easiest way to judge whether a curated skill feels worth adopting.

## Recommended Order

| Priority | Skill | Why start here |
|----------|-------|----------------|
| 1 | Frontend Design | Best first impression: it turns a vague brief into a visible interface and shows the quality of a curated skill immediately. |
| 2 | Webapp Testing | Best when you already have a running app and want the skill to interact with it directly in the browser. |
| 3 | PDF Processing | Best for document-heavy work when you need extraction, transformation, or form handling rather than UI or browser automation. |

## Curated Skills

| Skill | Source | Best for | Copyable command |
|-------|--------|----------|------------------|
| Frontend Design | [anthropics/skills](https://github.com/anthropics/skills/blob/main/skills/frontend-design/SKILL.md) | Turning a rough product brief into a visible UI result | `skillx run https://github.com/anthropics/skills/tree/main/skills/frontend-design "Design a distinctive landing page for a developer tool"` |
| Webapp Testing | [anthropics/skills](https://github.com/anthropics/skills/blob/main/skills/webapp-testing/SKILL.md) | Browser-driven checks against a running local app | `skillx run https://github.com/anthropics/skills/tree/main/skills/webapp-testing "Test my local web app at http://localhost:3000 for UI regressions and console errors"` |
| PDF Processing | [anthropics/skills](https://github.com/anthropics/skills/blob/main/skills/pdf/SKILL.md) | One-off extraction and transformation work for PDF-heavy tasks | `skillx run https://github.com/anthropics/skills/tree/main/skills/pdf "Extract the text, tables, and form fields from this PDF"` |

### <a id="frontend-design"></a>Frontend Design

Use this when you want the most convincing first run: the prompt is concrete, the output is visible, and the value of a reusable skill is easy to judge from the result.

### <a id="webapp-testing"></a>Webapp Testing

Use this when you already have a local app running and want a skill to exercise flows, report browser issues, and give you a faster QA loop.

### <a id="pdf-processing"></a>PDF Processing

Use this when the task is document-heavy and you need a focused capability right now without turning PDF handling into a permanent part of your setup.

## When to use Famous Skills

Use these when you want a proven workflow that already exists upstream and you do not need to inspect or modify the skill source. If you want to learn the repository's own skill patterns or build from a local template, use the official examples instead.

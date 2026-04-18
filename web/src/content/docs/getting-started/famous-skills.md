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

| Skill | 来源 | 用途 | 可复制命令 |
|-------|------|------|------------|
| Frontend Design | [anthropics/skills](https://github.com/anthropics/skills/blob/main/skills/frontend-design/SKILL.md) | 用于设计和实现高质量前端界面，适合页面、组件和应用 UI。 | `skillx run github:anthropics/skills/skills/frontend-design "Design a distinctive landing page for a developer tool"` |
| Webapp Testing | [anthropics/skills](https://github.com/anthropics/skills/blob/main/skills/webapp-testing/SKILL.md) | 用于用 Playwright 测试本地 Web 应用、排查 UI 问题并检查浏览器日志。 | `skillx run github:anthropics/skills/skills/webapp-testing "Test my local web app at http://localhost:3000 for UI regressions and console errors"` |
| PDF Processing | [anthropics/skills](https://github.com/anthropics/skills/blob/main/skills/pdf/SKILL.md) | 用于读取、提取、合并、拆分和生成 PDF，以及处理表单和扫描件。 | `skillx run github:anthropics/skills/skills/pdf "Extract the text, tables, and form fields from this PDF"` |

## When to use Famous Skills

Use these when you want a proven workflow that already exists upstream and you do not need to inspect or modify the skill source. If you want to learn the repository's own skill patterns or build from a local template, use the official examples instead.

# Run-First 文档站改版 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 让文档站完整承接 landing page 的 “一条命令，运行一个 GitHub skill” 主路径，同时移除 registry 对外叙事并强化 Examples 入口。

**Architecture:** 本次实现按 “入口重排 -> 主路径重写 -> 真实使用层补齐 -> registry 清理与事实修正” 四层推进。保留现有 Astro + Starlight 技术栈和大部分页面文件，优先通过 sidebar、导航、关键 docs 页面和 examples 入口完成信息架构切换，再补齐安全与使用层文案，最后清理 README / blog / 平台页面中的旧叙事。

**Tech Stack:** Astro 5、Starlight、Markdown docs、Astro components、npm scripts (`build`, `check:homepage`)

---

## 文件结构与责任边界

### 导航与站点配置

- Modify: `web/astro.config.mjs`
  - 重排 sidebar，把主入口改成 `Run a GitHub Skill`
  - 降级 `CLI Reference / Platforms / Agents`
  - 新增 `Famous Skills` 入口
- Modify: `web/src/components/nav.astro`
  - 修正顶部导航，让 `Examples` 指向正确入口
  - 确保 landing page 与 docs 入口语义一致

### 主路径页面

- Modify: `web/src/content/docs/getting-started/index.md`
  - 重写为 docs 真正入口页
- Modify: `web/src/content/docs/getting-started/installation.md`
  - 强化安装后下一步路径
- Modify: `web/src/content/docs/getting-started/first-run.md`
  - 改写为最短成功路径页
- Create: `web/src/content/docs/getting-started/famous-skills.md`
  - 新建 Famous Skills 精选页

### Examples 体系

- Modify: `web/src/content/docs/examples/overview.md`
  - 改成 Official Examples 入口页
- Modify: `web/src/content/docs/examples/code-review.md`
- Modify: `web/src/content/docs/examples/commit-message.md`
- Modify: `web/src/content/docs/examples/testing-guide.md`
- Modify: `web/src/content/docs/examples/hello-world.md`
- Modify: `web/src/content/docs/examples/name-poem.md`
  - 把现有 official examples 的定位写成“稳定、可维护、可学习”

### 真实使用层

- Modify: `web/src/content/docs/cli/run.md`
- Modify: `web/src/content/docs/cli/scan.md`
- Create: `web/src/content/docs/guides/manage-project-skills.md`
- Modify: `web/src/content/docs/cli/install.md`
- Modify: `web/src/content/docs/cli/init.md`
- Modify: `web/src/content/docs/cli/list.md`
- Modify: `web/src/content/docs/cli/update.md`
- Modify: `web/src/content/docs/cli/uninstall.md`
- Modify: `web/src/content/docs/agents/overview.md`

### 信任与清理

- Modify: `web/src/content/docs/security/overview.md`
- Modify: `web/src/content/docs/platforms/overview.md`
- Modify: `web/src/content/docs/platforms/git-hosts.md`
- Delete: `web/src/content/docs/platforms/skill-directories.md`
- Modify: `web/src/content/blog/introducing-skillx.md`
- Modify: `README.md`

### 验证

- Run: `npm run build` in `web/`
- Run: `npm run check:homepage` in `web/`
- Run: `rg -n "registry\\.skillx\\.run|Skill Directories|coming soon|planned command|planned for v0\\.4|curated registries and marketplaces" web/src/content README.md`

## Task 1: 重排 sidebar 与顶部导航

**Files:**
- Modify: `web/astro.config.mjs`
- Modify: `web/src/components/nav.astro`
- Test: `web/package.json`

- [ ] **Step 1: 先写失败检查，锁定旧导航仍然存在**

Run:

```bash
rg -n "Skill Directories|CLI Reference|href=\"/getting-started/first-run/\">Examples" \
  web/astro.config.mjs web/src/components/nav.astro
```

Expected:

```text
web/astro.config.mjs:33:          label: 'CLI Reference',
web/astro.config.mjs:53:            { label: 'Skill Directories', slug: 'platforms/skill-directories' },
web/src/components/nav.astro:10:      <a href="/getting-started/first-run/">Examples</a>
```

- [ ] **Step 2: 用 Run First 结构替换 Starlight sidebar**

在 `web/astro.config.mjs` 中，将现有 `sidebar` 配置改为下面这组结构：

```js
      sidebar: [
        {
          label: 'Run a GitHub Skill',
          items: [
            { label: 'Introduction', slug: 'getting-started' },
            { label: 'Installation', slug: 'getting-started/installation' },
            { label: 'First Run', slug: 'getting-started/first-run' },
            { label: 'Famous Skills', slug: 'getting-started/famous-skills' },
            { label: 'Official Examples', slug: 'examples/overview' },
          ],
        },
        {
          label: 'Use skillx in Real Work',
          items: [
            { label: 'Run Skills', slug: 'cli/run' },
            { label: 'Scan Skills', slug: 'cli/scan' },
            { label: 'Manage Project Skills', slug: 'guides/manage-project-skills' },
            { label: 'Agents', slug: 'agents/overview' },
          ],
        },
        {
          label: 'Trust & Security',
          items: [
            { label: 'Security Overview', slug: 'security/overview' },
            { label: 'Risk Levels', slug: 'security/risk-levels' },
            { label: 'Rules', slug: 'security/rules' },
            { label: 'CI Integration', slug: 'guides/ci-integration' },
          ],
        },
        {
          label: 'Build Skills',
          items: [
            { label: 'Writing Skills', slug: 'guides/writing-skills' },
            { label: 'Official Example Patterns', slug: 'examples/overview' },
            { label: 'Agent Adapters', slug: 'guides/agent-adapters' },
          ],
        },
        {
          label: 'Reference',
          items: [
            { label: 'CLI Flags', slug: 'reference/cli-flags' },
            { label: 'config.toml', slug: 'reference/config-toml' },
            { label: 'Manifest', slug: 'reference/manifest' },
            { label: 'Git Hosts', slug: 'platforms/git-hosts' },
            { label: 'Source URLs', slug: 'platforms/overview' },
          ],
        },
      ],
```

- [ ] **Step 3: 修正 landing 顶部导航的 docs / examples 指向**

在 `web/src/components/nav.astro` 中，把导航链接改为下面的结构：

```astro
    <div class="nav-links">
      <a href="/getting-started/">Docs</a>
      <a href="/getting-started/famous-skills/">Examples</a>
      <a href="https://github.com/skillx-run/skillx" class="gh-link">
        GitHub
      </a>
      <a href="#first-run" class="nav-cta">Run your first skill</a>
    </div>
```

- [ ] **Step 4: 运行构建，确认新 slug 仍未缺页**

Run:

```bash
npm run build
```

Workdir:

```bash
web
```

Expected:

```text
[build] Complete!
```

- [ ] **Step 5: 提交导航与 sidebar 重排**

```bash
git add web/astro.config.mjs web/src/components/nav.astro
git commit -m "feat(web): restructure docs navigation around run-first flow"
```

## Task 2: 重写 docs 入口、安装页与 First Run 页

**Files:**
- Modify: `web/src/content/docs/getting-started/index.md`
- Modify: `web/src/content/docs/getting-started/installation.md`
- Modify: `web/src/content/docs/getting-started/first-run.md`
- Test: `web/src/content/docs/getting-started/*.md`

- [ ] **Step 1: 先写失败检查，确认入口页仍是旧“产品介绍页”结构**

Run:

```bash
rg -n "What is skillx\\?|Why skillx\\?|This tutorial walks you through three common scenarios" \
  web/src/content/docs/getting-started/index.md \
  web/src/content/docs/getting-started/first-run.md
```

Expected:

```text
web/src/content/docs/getting-started/index.md:5:## What is skillx?
web/src/content/docs/getting-started/index.md:13:## Why skillx?
web/src/content/docs/getting-started/first-run.md:5:This tutorial walks you through three common scenarios
```

- [ ] **Step 2: 重写 docs 入口页，首屏先给安装与可运行命令**

将 `web/src/content/docs/getting-started/index.md` 替换为以下内容：

```md
---
title: Introduction to skillx
description: Install skillx, run a GitHub skill in one command, and choose your next path.
---

skillx is a CLI for one job first: **run a GitHub skill in one command**.

## Install skillx

```bash
curl -fsSL https://skillx.run/install.sh | sh
```

## Run a GitHub Skill

```bash
skillx run https://github.com/anthropics/skills/tree/main/skills/frontend-design \
  "Redesign the hero section of this landing page for higher conversion."
```

skillx fetches the skill, scans it before injection, launches your agent, and cleans the session up when you exit.

## Start Here

- [First Run](/getting-started/first-run/) — the shortest path from install to a real run
- [Famous Skills](/getting-started/famous-skills/) — high-signal GitHub skills worth trying now
- [Security Overview](/security/overview/) — understand the trust model before deeper adoption
- [Run Skills](/cli/run/) — learn the full `skillx run` workflow

## What to Do Next

### I want to try a real skill now

Go to [First Run](/getting-started/first-run/) or jump straight to [Famous Skills](/getting-started/famous-skills/).

### I want to use skillx in project workflows

Read [Manage Project Skills](/guides/manage-project-skills/) after your first run.

### I want to understand the safety model

Start with [Security Overview](/security/overview/) and then [Risk Levels](/security/risk-levels/).
```

- [ ] **Step 3: 重写安装页尾部分流，避免停在安装本身**

在 `web/src/content/docs/getting-started/installation.md` 中，将结尾的“Verify Installation / Data Directories / Uninstall”之后追加以下段落：

```md
## Next Step

Once `skillx --version` and `skillx agents` both work, do not stop at installation.

- Go to [First Run](/getting-started/first-run/) for the shortest end-to-end run
- Browse [Famous Skills](/getting-started/famous-skills/) if you want a stronger example than the local starter skills
- Read [Security Overview](/security/overview/) if you need to explain the trust model to your team
```

- [ ] **Step 4: 把 First Run 改成最短成功路径页**

将 `web/src/content/docs/getting-started/first-run.md` 重写为下面的结构：

```md
---
title: First Run
description: The shortest path from install to running a GitHub skill with skillx.
---

This page is optimized for one outcome: get from zero to a real GitHub skill run as fast as possible.

## 1. Install

```bash
curl -fsSL https://skillx.run/install.sh | sh
```

## 2. Run a Real GitHub Skill

```bash
skillx run https://github.com/anthropics/skills/tree/main/skills/frontend-design \
  "Redesign the hero section of this landing page for higher conversion. Keep the existing stack."
```

## 3. What You Should Expect

You should see a flow like:

```text
● Resolving source...
✓ Resolved: frontend-design
● Scanning for security issues...
● Detecting agents...
✓ Using agent: Codex
● Injecting skill...
● Launching agent...
```

## 4. What skillx is doing

1. Resolve the GitHub URL into a specific skill directory
2. Scan the skill before injection
3. Detect your installed agent
4. Inject the skill for this session
5. Clean up when the run ends

## 5. If you want the next best examples

- [Famous Skills](/getting-started/famous-skills/) — curated external GitHub skills
- [Official Examples](/examples/overview/) — stable first-party examples in this repo
- [Run Skills](/cli/run/) — the full command reference once the first run works
```

- [ ] **Step 5: 运行内容构建，确认 getting-started 三页都能生成**

Run:

```bash
npm run build
```

Workdir:

```bash
web
```

Expected:

```text
/getting-started/index.html
/getting-started/installation/index.html
/getting-started/first-run/index.html
```

- [ ] **Step 6: 提交主入口页重写**

```bash
git add \
  web/src/content/docs/getting-started/index.md \
  web/src/content/docs/getting-started/installation.md \
  web/src/content/docs/getting-started/first-run.md
git commit -m "feat(web): rewrite getting started docs around first run"
```

## Task 3: 新增 Famous Skills，并重构 Official Examples 入口

**Files:**
- Create: `web/src/content/docs/getting-started/famous-skills.md`
- Modify: `web/src/content/docs/examples/overview.md`
- Modify: `web/src/content/docs/examples/code-review.md`
- Modify: `web/src/content/docs/examples/commit-message.md`
- Modify: `web/src/content/docs/examples/testing-guide.md`
- Modify: `web/src/content/docs/examples/hello-world.md`
- Modify: `web/src/content/docs/examples/name-poem.md`

- [ ] **Step 1: 先写失败检查，确认 Famous Skills 页面还不存在，examples overview 还是旧分类**

Run:

```bash
test -f web/src/content/docs/getting-started/famous-skills.md; echo $?
rg -n "## Categories|### Showcase|### Starter|### Practical|### Security Demo" \
  web/src/content/docs/examples/overview.md
```

Expected:

```text
1
web/src/content/docs/examples/overview.md:7:## Categories
web/src/content/docs/examples/overview.md:9:### Showcase
web/src/content/docs/examples/overview.md:15:### Starter
```

- [ ] **Step 2: 新建 Famous Skills 精选页**

创建 `web/src/content/docs/getting-started/famous-skills.md`：

```md
---
title: Famous Skills
description: Curated GitHub skills you can run right now with skillx.
---

These are not first-party skillx examples. They are high-signal GitHub skills that make sense for engineers already using Codex, Claude Code, or Cursor.

## Featured Skills

### Frontend Design

- **Source:** `anthropics/skills`
- **Use it for:** landing pages, hero sections, conversion-focused UI revisions
- **Run it now:**

```bash
skillx run https://github.com/anthropics/skills/tree/main/skills/frontend-design \
  "Redesign the hero section of this landing page for higher conversion."
```

### Webapp Testing

- **Source:** `anthropics/skills`
- **Use it for:** local QA passes, signup flows, browser-based validation
- **Run it now:**

```bash
skillx run https://github.com/anthropics/skills/tree/main/skills/webapp-testing \
  "Test the signup flow on http://localhost:3000 and report broken states."
```

### PDF Processing

- **Source:** `anthropics/skills`
- **Use it for:** extracting tables, summarizing documents, one-off report analysis
- **Run it now:**

```bash
skillx run https://github.com/anthropics/skills/tree/main/skills/pdf \
  "Extract the tables from ./reports/q1.pdf and summarize key changes."
```

## Before You Run One

- Check [Security Overview](/security/overview/) if you need the trust model first
- Use [First Run](/getting-started/first-run/) if you have not run any GitHub skill yet
- Use [Official Examples](/examples/overview/) if you want stable first-party examples instead
```

- [ ] **Step 3: 把 examples overview 改成 Official Examples 入口页**

将 `web/src/content/docs/examples/overview.md` 替换为：

```md
---
title: Official Examples
description: Stable first-party skillx examples for learning, testing, and adaptation.
---

The pages in this section document the official examples shipped in the skillx repository. Use them when you want stable, maintained examples you can trust for learning and regression checks.

## How Official Examples differ from Famous Skills

- [Famous Skills](/getting-started/famous-skills/) are curated external GitHub skills chosen for immediate usefulness
- Official examples are maintained in this repository and should remain stable over time

## Start with these

| Example | Why start here |
|--------|-----------------|
| [Code Review](/examples/code-review) | Strong real-world review workflow with structured output |
| [Commit Message](/examples/commit-message) | Useful day-to-day skill for teams and solo developers |
| [Testing Guide](/examples/testing-guide) | Multi-file example with reusable guidance patterns |
| [Hello World](/examples/hello-world) | Minimal baseline for understanding the skill structure |
| [Name Poem](/examples/name-poem) | A fun, low-risk example for demos and onboarding |

## Use these pages for

- understanding how official examples are structured
- copying a stable example as the starting point for a custom skill
- validating changes to docs, scan behavior, or example flows
```

- [ ] **Step 4: 给每个 official example 页统一加“角色定位 + 下一步”段落**

在以下文件末尾追加统一结构：

```md
## Why this example exists

This is an official example maintained by the skillx repository. Treat it as a stable baseline, not as a directory of the most famous public skills.

## Next Steps

- [Famous Skills](/getting-started/famous-skills/) — try a higher-signal external GitHub skill
- [Official Examples](/examples/overview/) — compare other stable first-party examples
- [Writing Skills](/guides/writing-skills/) — use this example as the base for your own skill
```

Apply this to:

```text
web/src/content/docs/examples/code-review.md
web/src/content/docs/examples/commit-message.md
web/src/content/docs/examples/testing-guide.md
web/src/content/docs/examples/hello-world.md
web/src/content/docs/examples/name-poem.md
```

- [ ] **Step 5: 运行构建，确认 Famous Skills 与 Examples 页面全部生成**

Run:

```bash
npm run build
```

Workdir:

```bash
web
```

Expected:

```text
/getting-started/famous-skills/index.html
/examples/overview/index.html
/examples/code-review/index.html
/examples/commit-message/index.html
/examples/testing-guide/index.html
```

- [ ] **Step 6: 提交 examples 体系重构**

```bash
git add \
  web/src/content/docs/getting-started/famous-skills.md \
  web/src/content/docs/examples/overview.md \
  web/src/content/docs/examples/code-review.md \
  web/src/content/docs/examples/commit-message.md \
  web/src/content/docs/examples/testing-guide.md \
  web/src/content/docs/examples/hello-world.md \
  web/src/content/docs/examples/name-poem.md
git commit -m "feat(web): split docs examples into famous and official tracks"
```

## Task 4: 补齐真实使用层页面

**Files:**
- Modify: `web/src/content/docs/cli/run.md`
- Modify: `web/src/content/docs/cli/scan.md`
- Create: `web/src/content/docs/guides/manage-project-skills.md`
- Modify: `web/src/content/docs/cli/install.md`
- Modify: `web/src/content/docs/cli/init.md`
- Modify: `web/src/content/docs/cli/list.md`
- Modify: `web/src/content/docs/cli/update.md`
- Modify: `web/src/content/docs/cli/uninstall.md`
- Modify: `web/src/content/docs/agents/overview.md`

- [ ] **Step 1: 先写失败检查，锁定 facts drift 和缺失入口**

Run:

```bash
rg -n -- "--headless|--fail-on" web/src/content/docs/cli/run.md
rg -n -- "--full-auto|--sandbox=none" web/src/content/docs/agents/overview.md
test -f web/src/content/docs/guides/manage-project-skills.md; echo $?
```

Expected:

```text
1
web/src/content/docs/agents/overview.md:198:| OpenAI Codex | `--full-auto` |
web/src/content/docs/agents/overview.md:199:| Gemini CLI | `--sandbox=none` |
1
```

- [ ] **Step 2: 修正 `run` 页为真实使用页，并补齐真实 flags**

在 `web/src/content/docs/cli/run.md` 的 options 表中加入缺失参数，并将开头导语改成：

```md
Fetch a skill, scan it, inject it into your active agent, launch the run, and clean up afterward. Start here after your first successful GitHub skill run.
```

追加以下选项行：

```md
| `--headless` | — | — | Disable interactive prompts for CI or scripted runs |
| `--fail-on <level>` | — | — | Fail immediately when the scan reaches the given level |
```

并在文末添加：

```md
## After `run` starts working

- [Scan Skills](/cli/scan/) — enforce the same safety checks in automation
- [Manage Project Skills](/guides/manage-project-skills/) — move from one-off runs to repeatable team usage
- [Agents](/agents/overview/) — understand how skillx adapts to your installed agent
```

- [ ] **Step 3: 把 `scan` 页改成真实工作流入口**

在 `web/src/content/docs/cli/scan.md` 开头替换为：

```md
Use `skillx scan` when you want the safety decision without launching an agent. This is the command that turns the trust model into a repeatable workflow for teams and CI.
```

并在文末添加：

```md
## Next Steps

- [Security Overview](/security/overview/) — understand the scan-before-inject model
- [CI Integration](/guides/ci-integration/) — wire scans into pull requests and releases
- [Run Skills](/cli/run/) — move from audit to execution
```

- [ ] **Step 4: 新建 Manage Project Skills 汇总页**

创建 `web/src/content/docs/guides/manage-project-skills.md`：

```md
---
title: Manage Project Skills
description: Move from one-off runs to repeatable project skill workflows.
---

Use this section after `skillx run` is already working and you want shared, repeatable skill usage in a project.

## Core workflow

1. `skillx init` — create `skillx.toml`
2. `skillx install` — persist selected skills
3. `skillx list` — inspect current state
4. `skillx update` — refresh installed skills
5. `skillx uninstall` — remove skills you no longer need

## Read in this order

- [skillx init](/cli/init/)
- [skillx install](/cli/install/)
- [skillx list](/cli/list/)
- [skillx update](/cli/update/)
- [skillx uninstall](/cli/uninstall/)
```

- [ ] **Step 5: 给 project skill 相关命令页统一补“返回主路径”链接**

在以下文件末尾追加：

```md
## Related Docs

- [Manage Project Skills](/guides/manage-project-skills/) — the full project workflow
- [Run Skills](/cli/run/) — use one-off runs before you decide to persist anything
- [Official Examples](/examples/overview/) — stable examples you can install into a project
```

Apply to:

```text
web/src/content/docs/cli/install.md
web/src/content/docs/cli/init.md
web/src/content/docs/cli/list.md
web/src/content/docs/cli/update.md
web/src/content/docs/cli/uninstall.md
```

- [ ] **Step 6: 修正 Agents 概览页里的自动批准参数与定位**

在 `web/src/content/docs/agents/overview.md` 中：

1. 将开头导语改为：

```md
Use this page after your first run works and you want to understand how skillx adapts to Codex, Claude Code, Cursor, and other supported agents.
```

2. 将 auto-approve 表格改成：

```md
| Agent | Auto-approve Flag |
|-------|-----------|
| Claude Code | `--dangerously-skip-permissions` |
| OpenAI Codex | `--yolo` |
| Gemini CLI | `--yolo` |
| Amp | `--dangerously-allow-all` |
| All others | Not supported |
```

3. 将 Amp 路径改成：

```md
| Amp | `~/.config/agents/skills/<name>/` | `.agents/skills/<name>/` |
```

- [ ] **Step 7: 运行构建并检查 Use skillx in Real Work 主路径是否闭环**

Run:

```bash
npm run build && rg -n "Manage Project Skills|After `run` starts working|Related Docs" \
  src/content/docs/cli src/content/docs/guides src/content/docs/agents
```

Workdir:

```bash
web
```

Expected:

```text
[build] Complete!
src/content/docs/guides/manage-project-skills.md:1:---
src/content/docs/cli/run.md:2:title: "skillx run"
src/content/docs/cli/install.md:2:title: "skillx install"
```

- [ ] **Step 8: 提交真实使用层文档**

```bash
git add \
  web/src/content/docs/cli/run.md \
  web/src/content/docs/cli/scan.md \
  web/src/content/docs/guides/manage-project-skills.md \
  web/src/content/docs/cli/install.md \
  web/src/content/docs/cli/init.md \
  web/src/content/docs/cli/list.md \
  web/src/content/docs/cli/update.md \
  web/src/content/docs/cli/uninstall.md \
  web/src/content/docs/agents/overview.md
git commit -m "feat(web): add real-work docs flow for run scan and project skills"
```

## Task 5: 重写安全页并清理 registry / 平台旧叙事

**Files:**
- Modify: `web/src/content/docs/security/overview.md`
- Modify: `web/src/content/docs/platforms/overview.md`
- Modify: `web/src/content/docs/platforms/git-hosts.md`
- Delete: `web/src/content/docs/platforms/skill-directories.md`
- Modify: `web/src/content/blog/introducing-skillx.md`
- Modify: `README.md`

- [ ] **Step 1: 先写失败检查，锁定 registry 与旧平台叙事仍然存在**

Run:

```bash
rg -n "registry\\.skillx\\.run|Skill Directories|planned for v0\\.4|coming soon|curated registries and marketplaces" \
  web/src/content/docs/platforms \
  web/src/content/blog/introducing-skillx.md \
  README.md
```

Expected:

```text
README.md:81:| Skill Directories | 10 supported platforms |
web/src/content/blog/introducing-skillx.md:72:- **v0.4** — `registry.skillx.run` for skill discovery
web/src/content/docs/platforms/skill-directories.md:87:The skillx registry API (planned for v0.4+) will provide:
```

- [ ] **Step 2: 把 Security Overview 改成 trust 入口页**

将 `web/src/content/docs/security/overview.md` 的开头替换为：

```md
## Why this page exists

Run-first only works if users can quickly understand the trust boundary. `skillx` is not promising that every public skill is safe. It is promising that every skill is scanned before injection, clearly gated by risk level, and cleaned up after the run.
```

并将 “Defense Layers” 标题前增加：

```md
## The trust model in one sentence

`scan before inject, gate before launch, clean after exit`
```

- [ ] **Step 3: 重写平台总览页，只保留真实 source URL 能力**

把 `web/src/content/docs/platforms/overview.md` 中的 “Skill Directory URL” 章节、支持表里的 `Skill Directories` 行、末尾指向目录市场的链接删除，并将标题后的第一段改成：

```md
This page documents the real source URL formats that skillx resolves today: local paths, Git hosts, archives, and shorthand prefixes such as `github:` and `gist:`.
```

支持表替换为：

```md
| Platform | Status | Source Format |
|----------|--------|---------------|
| Local filesystem | Supported | `./path`, `/path`, `~/path` |
| GitHub | Supported | `github:owner/repo/path` or URL |
| GitLab | Supported | URL |
| Bitbucket | Supported | URL |
| Gitea / Codeberg | Supported | URL |
| GitHub Gist | Supported | `gist:id` or URL |
| SourceHut | Supported | URL |
| HuggingFace | Supported | URL |
| Archive | Supported | `.zip` / `.tar.gz` URL |
```

- [ ] **Step 4: 删除 Skill Directories 页，并同步清理 README / blog**

执行：

```bash
rm web/src/content/docs/platforms/skill-directories.md
```

同时修改：

`README.md`

```md
## Supported Sources

9 source types with smart URL recognition across major Git hosts and archive formats:
```

并删除来源表中的 `Skill Directories` 行。

`web/src/content/blog/introducing-skillx.md`

将：

```md
- **v0.4** — `registry.skillx.run` for skill discovery
```

替换为：

```md
- **v0.4** — continue refining the run-first workflow, more examples, and stronger docs
```

- [ ] **Step 5: 调整 Git Hosts 页尾链接，避免再把用户导回目录市场叙事**

在 `web/src/content/docs/platforms/git-hosts.md` 文末追加：

```md
## Next Steps

- [Source URLs](/platforms/overview/) — see the full list of source formats supported today
- [Run Skills](/cli/run/) — move from source recognition to actual execution
- [Famous Skills](/getting-started/famous-skills/) — try curated GitHub skills now
```

- [ ] **Step 6: 运行清理验证，确认 registry 叙事只留在 superpowers 文档中**

Run:

```bash
rg -n "registry\\.skillx\\.run|Skill Directories|coming soon|planned command|planned for v0\\.4|curated registries and marketplaces" \
  web/src/content README.md web/src/pages web/src/components
```

Expected:

```text
该命令没有任何匹配输出，并以退出码 1 结束。
```

- [ ] **Step 7: 运行完整构建与首页 smoke check**

Run:

```bash
npm run build
npm run check:homepage
```

Workdir:

```bash
web
```

Expected:

```text
[build] Complete!
Homepage smoke checks passed
```

- [ ] **Step 8: 提交安全与清理收尾改动**

```bash
git add \
  web/src/content/docs/security/overview.md \
  web/src/content/docs/platforms/overview.md \
  web/src/content/docs/platforms/git-hosts.md \
  web/src/content/blog/introducing-skillx.md \
  README.md
git rm web/src/content/docs/platforms/skill-directories.md
git commit -m "feat(web): remove registry messaging from public docs"
```

## Task 6: 最终校验与交接

**Files:**
- Modify: `web/astro.config.mjs`
- Modify: `web/src/content/docs/**/*`
- Modify: `web/src/components/nav.astro`
- Modify: `README.md`
- Modify: `web/src/content/blog/introducing-skillx.md`

- [ ] **Step 1: 跑完整构建并保存成功输出**

Run:

```bash
npm run build
```

Workdir:

```bash
web
```

Expected:

```text
[build] Complete!
```

- [ ] **Step 2: 跑首页 smoke check**

Run:

```bash
npm run check:homepage
```

Workdir:

```bash
web
```

Expected:

```text
Homepage smoke checks passed
```

- [ ] **Step 3: 检查主路径与清理约束**

Run:

```bash
rg -n "Run a GitHub Skill|Famous Skills|Official Examples|Manage Project Skills|Trust & Security" \
  web/astro.config.mjs web/src/content/docs
rg -n "registry\\.skillx\\.run|Skill Directories|coming soon|planned command|planned for v0\\.4" \
  web/src/content README.md web/src/pages web/src/components
```

Expected:

```text
web/astro.config.mjs:25:          label: 'Run a GitHub Skill',
web/src/content/docs/getting-started/famous-skills.md:2:title: Famous Skills
web/src/content/docs/examples/overview.md:2:title: Official Examples
```

第二条命令预期没有任何匹配输出，并以退出码 1 结束。

- [ ] **Step 4: 查看最终 diff，确认没有误删无关内容**

Run:

```bash
git status --short
git diff --stat HEAD~5..HEAD
```

Expected:

```text
 M README.md
 M web/astro.config.mjs
 M web/src/components/nav.astro
 M web/src/content/blog/introducing-skillx.md
```

并且 diff 只覆盖本计划列出的文件。

- [ ] **Step 5: 汇总验证结果并准备交接说明**

在最终说明中明确写出：

```text
1. 新 docs 主路径：Run a GitHub Skill -> Use skillx in Real Work -> Trust & Security
2. Famous Skills / Official Examples 已分层
3. registry 对外叙事已移除
4. 已执行 npm run build 与 npm run check:homepage
```

## 计划自检

### Spec 覆盖情况

- `Run First` 主路径：由 Task 1、Task 2 完成
- `Examples` 双层体系：由 Task 3 完成
- `Use skillx in Real Work`：由 Task 4 完成
- `Trust & Security`：由 Task 5 完成
- registry 清理：由 Task 5 完成
- README / blog 同步：由 Task 5 完成
- 完整验证：由 Task 6 完成

### Placeholder 扫描

本计划未使用 `TODO`、`TBD`、`implement later`、`add appropriate handling` 等占位描述；所有改动均指向明确文件、片段和命令。

### 类型与命名一致性

- 统一使用 `Run a GitHub Skill` 作为 docs 主入口名
- 统一使用 `Famous Skills` / `Official Examples`
- 统一使用 `Manage Project Skills`
- 所有新增 slug 均与 sidebar 配置一致

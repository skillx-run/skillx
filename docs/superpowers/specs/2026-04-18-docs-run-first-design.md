# 文档站改版设计：对齐 Run First 主路径

## 背景

当前 `skillx.run` 的 landing page 已经明确围绕一句核心承诺展开：

`一条命令，运行一个 GitHub skill`

最近的首页改版已经把产品价值收敛到 first-run、低摩擦、先扫描再注入、默认临时清理这几个点上。但文档站仍然保留了更早期的组织方式：栏目平铺、CLI 参考优先、Platforms/Agents 独立成大区块、Examples 偏演示与教学而非转化。

这造成两个问题：

1. 用户从 landing page 进入 docs 后，主路径断裂，无法自然沿着“先运行一个 skill，再理解工具”的路径前进。
2. 文档站中仍混有 registry / planned API / future workflow 等内容，和当前“先把工具打磨好”的阶段目标不一致。

本次设计的目标不是重新定义 landing page，而是让文档站完整承接现有 landing page 逻辑。

## 目标

本次改版要达到以下目标：

1. 让 docs 的默认路径与 landing page 的承诺一致：先跑一个 GitHub skill，再理解 skillx 的安全边界和使用方式。
2. 让已经在使用 `Codex / Claude Code / Cursor` 的工程师能在最短路径内完成 first run。
3. 强化 Examples 的转化作用，提供更“立即可用、有名、有趣”的入口。
4. 保留完整工具文档与参考资料，但将其降到第二层，不再主导首屏路径。
5. 移除 registry 相关对外叙事，避免当前产品阶段与文档承诺错位。

## 非目标

本次改版不包含以下目标：

1. 不重新定义 landing page 主叙事。
2. 不在本轮引入新的 registry、search、publish marketplace 叙事。
3. 不在第一轮里扩展大量新页面，避免再次把文档站做回“内容很多但路径不清楚”的结构。
4. 不处理与文档站无关的 CLI 行为改动。

## 核心设计原则

### 1. Run First

文档站的默认问题不应是“skillx 是什么”，而应是：

`我如何最快运行一个 GitHub skill？`

因此 docs 首页与 sidebar 首段都必须围绕 first run 展开。

### 2. 主副轨分离

文档站采用双层结构：

- 主轨：引导用户完成 first run，看到 examples，理解安全机制，继续进入真实使用。
- 副轨：提供 CLI、Agents、Reference、Authoring 等完整资料。

主轨承担转化和承接 landing page 的职责；副轨承担查阅和深度使用职责。

### 3. Examples 不是点缀，而是入口

Examples 不再作为边缘栏目存在，而是成为 docs 主入口的一部分。Examples 的存在目标不是“展示 skillx 支持 example”，而是“让用户现在就想跑一个 skill”。

### 4. 当前阶段只强调已经成立的事情

文档只描述今天已经成立、已经支持、已经可验证的能力。规划中的 registry、search、API、目录发布流程全部移出对外主文档。

## 新的信息架构

建议将文档站一级结构重组为以下五组：

### 1. Run a GitHub Skill

这是文档站主入口，直接承接 landing page。

包含页面：

- `Introduction`
- `Installation`
- `First Run`
- `Famous Skills`
- `Official Examples`

职责：

- 让用户完成 first run
- 提供高信号 example
- 建立“skillx 是拿来直接跑 skill 的”心智

### 2. Use skillx in Real Work

这是 first run 之后的继续使用层。

包含页面：

- `Run Skills`
- `Scan Skills`
- `Manage Project Skills`
- `Agents`

职责：

- 解释如何把 skillx 用到真实项目中
- 承接 `run / scan / init / install / update / list / uninstall`
- 让用户从“一次尝试”走向“稳定使用”

### 3. Trust & Security

这是 landing page trust 逻辑在 docs 中的承接区。

包含页面：

- `Security Overview`
- `Risk Levels`
- `Rules`
- `CI Integration`

职责：

- 解释为什么 skillx 值得信任
- 解释 scan-before-inject 与风险门控
- 支撑用户做安全判断

### 4. Build Skills

这是作者向内容，但降级到较后位置。

包含页面：

- `Writing Skills`
- `Example Patterns`
- `Agent Adapters`

职责：

- 服务 skill 作者
- 保留 first-party 模板和写作基线
- 不抢占 first-run 主路径

### 5. Reference

这是纯查阅区。

包含页面：

- `CLI Flags`
- `config.toml`
- `Manifest`
- `Git Hosts / Source URLs`

职责：

- 收纳不适合作为主路径入口的高密度资料
- 保持可查，但不主导导航

## 现有栏目调整策略

### Getting Started

保留 `Getting Started` 内容，但不继续以“产品介绍页”方式存在。

改造方向：

- `Introduction`：收敛为简短入口页，不再展开过多概念背景
- `Installation`：保留安装方式，但提高复制命令与下一步跳转的显著性
- `First Run`：重写为最短成功路径页

### CLI Reference

不删除 CLI 文档，但不再把“CLI Reference”作为导航中最强的一级心智。

处理策略：

- `run`、`scan` 优先提升到真实使用路径中
- `install / init / update / list / uninstall` 合并到 “Manage Project Skills”
- 其他高密度命令细节保留在 Reference 或次级页面中

### Agents

不再保留为独立大世界观栏目。

处理策略：

- 将 Agents 从“产品认知入口”降级为“运行环境说明”
- 保留兼容性说明、注入路径、生命周期模式
- 将其纳入 `Use skillx in Real Work`

### Platforms

整组降级并拆解。

处理策略：

- `Git Hosts` 与 source URL 相关内容并入 Reference
- `Skill Directories` 中的 registry、planned API、directory publishing 等内容移除
- 不再保留 Platforms 作为首页级认知入口

## Examples 体系设计

Examples 采用双层结构，但两层承担不同职责。

### Famous Skills

职责：转化。

它面向的问题是：

`给我一个现在就值得跑的 GitHub skill`

选择标准：

1. 一条命令即可尝试
2. 结果直观
3. 来源知名或技能本身有辨识度
4. 适合已有 agent 使用习惯的工程师

内容模板建议统一为：

- 名称
- 来源仓库 / 作者
- 这个 skill 解决什么问题
- 推荐使用场景
- 可直接复制的 `skillx run ...`
- 要求与前置条件
- 预期结果
- 原始 skill 链接

该栏目优先提供 `6-10` 个精选，而不是长列表。

### Official Examples

职责：稳定教学、官方基线、长期维护。

它面向的问题是：

`我想看稳定的 first-party example，并理解 skill 应该怎么写`

建议方向：

- 保留当前官方样例中真正有用的部分
- 弱化 toy 型示例
- 补充工作型 example

建议优先保留或新增的 example 类型：

- `code-review`
- `commit-message`
- `test-writer`
- `bug-triage`
- `refactor-planner`
- `release-notes`
- `docs-summarizer`
- `landing-page-critique`

### 双层关系

`Famous Skills` 放在前面，负责“马上想试”；`Official Examples` 放在后面，负责“长期可依赖、可学习、可维护”。

两者不混排，不共享一个松散 overview。

## 页面级重写方向

### docs 首页 / Introduction

改造目标：成为 docs 真正入口，而不是概念介绍页。

页面结构建议：

1. 安装命令
2. 一条可直接复制的 GitHub skill 命令
3. 两条立即分流：
   - 看 Famous Skills
   - 理解安全机制
4. 后续路径：
   - `Run`
   - `Scan`
   - `Manage Project Skills`

### First Run

改造目标：成为最短成功路径页。

页面必须回答：

1. 我复制哪条命令？
2. 运行后会看到什么？
3. skillx 在背后做了什么？
4. 如果成功，下一步去哪？

避免在这一页里塞入过多延伸知识。

### Examples Overview

改造目标：成为 examples 总入口。

页面结构建议：

1. `Famous Skills`
2. `Official Examples`
3. 说明两者区别
4. 给出“我应该从哪里开始”的推荐路径

### Security Overview

改造目标：承接 trust 逻辑，而不是仅仅罗列规则。

重点排序建议：

1. 为什么 skill 有风险
2. skillx 如何在注入前扫描
3. 风险门控如何工作
4. skillx 不能替代什么
5. 再跳去 `Risk Levels` 和 `Rules`

### Run / Scan

改造目标：从纯参考页变成“真实使用页”。

每页都应包含：

- 什么时候用
- 最常见用法
- 常见组合
- 下一步跳转

## registry 相关内容移除范围

本轮移除范围是“所有对外文档”，包括但不限于：

- docs 站内的 registry / planned API / future marketplace 叙事
- blog 中不再适配当前阶段的 roadmap 表述
- README 等对外文档中的 registry 承诺性表述

保留原则：

- 如某目录平台 URL 已经真实支持，可作为 source URL 能力的一部分保留
- 但不再以“registry 即将到来”或“目录发布流程即将形成”的方式表达

## 推进顺序

### 第一批：先打通主路径

优先改动：

1. sidebar
2. docs 首页 / `getting-started/index`
3. `getting-started/first-run`
4. `examples/overview`
5. docs 导航入口链接

目标：

- 让 landing page 进入 docs 后的路径第一次成立

### 第二批：补齐真实使用层

优先改动：

1. `run`
2. `scan`
3. `security/overview`
4. `agents/overview` 或替代页
5. `install / init / update / list / uninstall` 的整理页

目标：

- 让用户在 first run 后自然进入更深使用

### 第三批：清理外围与修复一致性

优先改动：

1. registry 相关内容清理
2. blog / README 同步
3. 文档事实漂移修复
4. 不再适合当前结构的旧页面降级、合并或改名

目标：

- 清理旧叙事残留
- 保持对外表达一致

## 验收标准

文档站改版完成后，应满足以下标准：

1. 用户从 landing page 进入 docs 后，能在主导航前几项里直接完成 first run。
2. `Examples` 成为主路径的一部分，而不是边缘栏目。
3. registry / planned API / future marketplace 叙事不再出现在对外主文档中。
4. CLI、Agents、Security、Reference 仍完整保留，但不再主导首屏路径。
5. docs 首页、First Run、Examples Overview 之间形成清晰闭环。
6. 文档结构能明显服务“已经在用 agent 的工程师”这一优先用户群。

## 风险与取舍

### 风险 1：Examples 过强，工具被背景化

应对：

- 通过 “Run First -> Why skillx -> Real Work” 的顺序保持工具主线
- 不让 examples 变成单纯内容目录

### 风险 2：外部 Famous Skills 失效或变动

应对：

- 保持精选数量克制
- Famous 与 Official 双层分离
- Official Examples 始终提供稳定 fallback

### 风险 3：一次性改动过大导致结构再度混乱

应对：

- 严格按三批推进
- 第一轮先完成路径重排与入口重写，不追求一轮做完所有内容

## 结论

本次文档站改版的本质，不是“补更多文档”，而是：

`让 docs 成为 landing page 承诺的延伸，而不是另一套平铺的技术目录。`

当用户记住的是“一条命令，运行一个 GitHub skill”时，docs 就必须首先服务这件事，然后再服务查阅、深度使用和创作扩展。

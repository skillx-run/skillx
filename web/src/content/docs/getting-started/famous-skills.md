---
title: Famous Skills
description: Curated external Agent Skills you can run immediately with skillx.
---

Famous Skills are curated external Agent Skills from well-known repositories. They are ranked for immediate usefulness: start with the one that matches your current task, then move to the others when you need a narrower workflow.

If you only try one, start with **Frontend Design**. It still gives the strongest end-to-end first run: a concrete prompt produces visible output, and it is the easiest way to judge whether a curated skill feels worth adopting.

## Recommended Order

| Priority | Skill | Why start here |
|----------|-------|----------------|
| 1 | Frontend Design | Best first impression: it turns a vague brief into a visible interface and shows the quality of a curated skill immediately. |
| 2 | Webapp Testing | Best when you already have a running app and want the skill to interact with it directly in the browser. |
| 3 | MCP Builder | Best when your next step is building tools and API integrations instead of screens. |
| 4 | Claude API | Best when you are already shipping Anthropic features and want stronger API defaults fast. |
| 5 | PDF Processing | Best for document-heavy work when you need extraction, transformation, or form handling rather than UI or browser automation. |

## Curated Skills

<div class="famous-skills-table-shell">
  <table class="famous-skills-table">
    <thead>
      <tr>
        <th>Skill</th>
        <th>Best for</th>
        <th>Copyable command</th>
      </tr>
    </thead>
    <tbody>
      <tr>
        <td>
          <strong>Frontend Design</strong>
          <div class="famous-skill-links">
            <a href="https://github.com/anthropics/skills/blob/main/skills/frontend-design/SKILL.md">SKILL.md</a>
            <a href="https://github.com/anthropics/skills/tree/main/skills/frontend-design">Directory</a>
          </div>
        </td>
        <td>Turning a rough product brief into a visible UI result</td>
        <td class="famous-command-cell">
          <div class="famous-command-stack">
            <div class="famous-command-scroll">
              <code>skillx run github:anthropics/skills/skills/frontend-design "Design a distinctive landing page for a developer tool"</code>
            </div>
            <div class="famous-command-actions">
              <button type="button" class="famous-copy-button" data-copy='skillx run github:anthropics/skills/skills/frontend-design "Design a distinctive landing page for a developer tool"' onclick="navigator.clipboard.writeText(this.dataset.copy); const label = this.querySelector('span'); const original = label.textContent; label.textContent = 'Copied'; setTimeout(() => label.textContent = original, 1200);">
                <span>Copy</span>
              </button>
              <a class="famous-source-link" href="https://github.com/anthropics/skills/blob/main/skills/frontend-design/SKILL.md">View source</a>
            </div>
          </div>
        </td>
      </tr>
      <tr>
        <td>
          <strong>Setup skillx</strong>
          <div class="famous-skill-links">
            <a href="https://github.com/skillx-run/skillx/blob/main/examples/skills/setup-skillx/SKILL.md">SKILL.md</a>
            <a href="https://github.com/skillx-run/skillx/tree/main/examples/skills/setup-skillx">Directory</a>
          </div>
        </td>
        <td>Advertising your own skill so others can try it with skillx</td>
        <td class="famous-command-cell">
          <div class="famous-command-stack">
            <div class="famous-command-scroll">
              <code>skillx run github:skillx-run/skillx/examples/skills/setup-skillx</code>
            </div>
            <div class="famous-command-actions">
              <button type="button" class="famous-copy-button" data-copy="skillx run github:skillx-run/skillx/examples/skills/setup-skillx" onclick="navigator.clipboard.writeText(this.dataset.copy); const label = this.querySelector('span'); const original = label.textContent; label.textContent = 'Copied'; setTimeout(() => label.textContent = original, 1200);">
                <span>Copy</span>
              </button>
              <a class="famous-source-link" href="https://github.com/skillx-run/skillx/blob/main/examples/skills/setup-skillx/SKILL.md">View source</a>
            </div>
          </div>
        </td>
      </tr>
      <tr>
        <td>
          <strong>Webapp Testing</strong>
          <div class="famous-skill-links">
            <a href="https://github.com/anthropics/skills/blob/main/skills/webapp-testing/SKILL.md">SKILL.md</a>
            <a href="https://github.com/anthropics/skills/tree/main/skills/webapp-testing">Directory</a>
          </div>
        </td>
        <td>Browser-driven checks against a running local app</td>
        <td class="famous-command-cell">
          <div class="famous-command-stack">
            <div class="famous-command-scroll">
              <code>skillx run github:anthropics/skills/skills/webapp-testing "Test my local web app at http://localhost:3000 for UI regressions and console errors"</code>
            </div>
            <div class="famous-command-actions">
              <button type="button" class="famous-copy-button" data-copy='skillx run github:anthropics/skills/skills/webapp-testing "Test my local web app at http://localhost:3000 for UI regressions and console errors"' onclick="navigator.clipboard.writeText(this.dataset.copy); const label = this.querySelector('span'); const original = label.textContent; label.textContent = 'Copied'; setTimeout(() => label.textContent = original, 1200);">
                <span>Copy</span>
              </button>
              <a class="famous-source-link" href="https://github.com/anthropics/skills/blob/main/skills/webapp-testing/SKILL.md">View source</a>
            </div>
          </div>
        </td>
      </tr>
      <tr>
        <td>
          <strong>PDF Processing</strong>
          <div class="famous-skill-links">
            <a href="https://github.com/anthropics/skills/blob/main/skills/pdf/SKILL.md">SKILL.md</a>
            <a href="https://github.com/anthropics/skills/tree/main/skills/pdf">Directory</a>
          </div>
        </td>
        <td>One-off extraction and transformation work for PDF-heavy tasks</td>
        <td class="famous-command-cell">
          <div class="famous-command-stack">
            <div class="famous-command-scroll">
              <code>skillx run github:anthropics/skills/skills/pdf "Extract the text, tables, and form fields from this PDF"</code>
            </div>
            <div class="famous-command-actions">
              <button type="button" class="famous-copy-button" data-copy='skillx run github:anthropics/skills/skills/pdf "Extract the text, tables, and form fields from this PDF"' onclick="navigator.clipboard.writeText(this.dataset.copy); const label = this.querySelector('span'); const original = label.textContent; label.textContent = 'Copied'; setTimeout(() => label.textContent = original, 1200);">
                <span>Copy</span>
              </button>
              <a class="famous-source-link" href="https://github.com/anthropics/skills/blob/main/skills/pdf/SKILL.md">View source</a>
            </div>
          </div>
        </td>
      </tr>
      <tr>
        <td>
          <strong>MCP Builder</strong>
          <div class="famous-skill-links">
            <a href="https://github.com/anthropics/skills/blob/main/skills/mcp-builder/SKILL.md">SKILL.md</a>
            <a href="https://github.com/anthropics/skills/tree/main/skills/mcp-builder">Directory</a>
          </div>
        </td>
        <td>Scaffolding an MCP server around a real API or service</td>
        <td class="famous-command-cell">
          <div class="famous-command-stack">
            <div class="famous-command-scroll">
              <code>skillx run github:anthropics/skills/skills/mcp-builder "Create an MCP server plan and starter implementation for a GitHub Issues integration"</code>
            </div>
            <div class="famous-command-actions">
              <button type="button" class="famous-copy-button" data-copy='skillx run github:anthropics/skills/skills/mcp-builder "Create an MCP server plan and starter implementation for a GitHub Issues integration"' onclick="navigator.clipboard.writeText(this.dataset.copy); const label = this.querySelector('span'); const original = label.textContent; label.textContent = 'Copied'; setTimeout(() => label.textContent = original, 1200);">
                <span>Copy</span>
              </button>
              <a class="famous-source-link" href="https://github.com/anthropics/skills/blob/main/skills/mcp-builder/SKILL.md">View source</a>
            </div>
          </div>
        </td>
      </tr>
      <tr>
        <td>
          <strong>Claude API</strong>
          <div class="famous-skill-links">
            <a href="https://github.com/anthropics/skills/blob/main/skills/claude-api/SKILL.md">SKILL.md</a>
            <a href="https://github.com/anthropics/skills/tree/main/skills/claude-api">Directory</a>
          </div>
        </td>
        <td>Building or debugging Anthropic API integrations</td>
        <td class="famous-command-cell">
          <div class="famous-command-stack">
            <div class="famous-command-scroll">
              <code>skillx run github:anthropics/skills/skills/claude-api "Add prompt caching and streaming to my Anthropic API app and explain the code changes"</code>
            </div>
            <div class="famous-command-actions">
              <button type="button" class="famous-copy-button" data-copy='skillx run github:anthropics/skills/skills/claude-api "Add prompt caching and streaming to my Anthropic API app and explain the code changes"' onclick="navigator.clipboard.writeText(this.dataset.copy); const label = this.querySelector('span'); const original = label.textContent; label.textContent = 'Copied'; setTimeout(() => label.textContent = original, 1200);">
                <span>Copy</span>
              </button>
              <a class="famous-source-link" href="https://github.com/anthropics/skills/blob/main/skills/claude-api/SKILL.md">View source</a>
            </div>
          </div>
        </td>
      </tr>
      <tr>
        <td>
          <strong>Canvas Design</strong>
          <div class="famous-skill-links">
            <a href="https://github.com/anthropics/skills/blob/main/skills/canvas-design/SKILL.md">SKILL.md</a>
            <a href="https://github.com/anthropics/skills/tree/main/skills/canvas-design">Directory</a>
          </div>
        </td>
        <td>Designing polished visual assets and canvas-style layouts</td>
        <td class="famous-command-cell">
          <div class="famous-command-stack">
            <div class="famous-command-scroll">
              <code>skillx run github:anthropics/skills/skills/canvas-design "Create a bold product announcement poster in a modern editorial style"</code>
            </div>
            <div class="famous-command-actions">
              <button type="button" class="famous-copy-button" data-copy='skillx run github:anthropics/skills/skills/canvas-design "Create a bold product announcement poster in a modern editorial style"' onclick="navigator.clipboard.writeText(this.dataset.copy); const label = this.querySelector('span'); const original = label.textContent; label.textContent = 'Copied'; setTimeout(() => label.textContent = original, 1200);">
                <span>Copy</span>
              </button>
              <a class="famous-source-link" href="https://github.com/anthropics/skills/blob/main/skills/canvas-design/SKILL.md">View source</a>
            </div>
          </div>
        </td>
      </tr>
      <tr>
        <td>
          <strong>DOCX</strong>
          <div class="famous-skill-links">
            <a href="https://github.com/anthropics/skills/blob/main/skills/docx/SKILL.md">SKILL.md</a>
            <a href="https://github.com/anthropics/skills/tree/main/skills/docx">Directory</a>
          </div>
        </td>
        <td>Creating or editing polished .docx deliverables</td>
        <td class="famous-command-cell">
          <div class="famous-command-stack">
            <div class="famous-command-scroll">
              <code>skillx run github:anthropics/skills/skills/docx "Create a polished project update memo as a .docx file with headings and a summary table"</code>
            </div>
            <div class="famous-command-actions">
              <button type="button" class="famous-copy-button" data-copy='skillx run github:anthropics/skills/skills/docx "Create a polished project update memo as a .docx file with headings and a summary table"' onclick="navigator.clipboard.writeText(this.dataset.copy); const label = this.querySelector('span'); const original = label.textContent; label.textContent = 'Copied'; setTimeout(() => label.textContent = original, 1200);">
                <span>Copy</span>
              </button>
              <a class="famous-source-link" href="https://github.com/anthropics/skills/blob/main/skills/docx/SKILL.md">View source</a>
            </div>
          </div>
        </td>
      </tr>
      <tr>
        <td>
          <strong>XLSX</strong>
          <div class="famous-skill-links">
            <a href="https://github.com/anthropics/skills/blob/main/skills/xlsx/SKILL.md">SKILL.md</a>
            <a href="https://github.com/anthropics/skills/tree/main/skills/xlsx">Directory</a>
          </div>
        </td>
        <td>Cleaning, analyzing, and restructuring spreadsheet workbooks</td>
        <td class="famous-command-cell">
          <div class="famous-command-stack">
            <div class="famous-command-scroll">
              <code>skillx run github:anthropics/skills/skills/xlsx "Clean this sales workbook, add summary formulas, and prepare it for review"</code>
            </div>
            <div class="famous-command-actions">
              <button type="button" class="famous-copy-button" data-copy='skillx run github:anthropics/skills/skills/xlsx "Clean this sales workbook, add summary formulas, and prepare it for review"' onclick="navigator.clipboard.writeText(this.dataset.copy); const label = this.querySelector('span'); const original = label.textContent; label.textContent = 'Copied'; setTimeout(() => label.textContent = original, 1200);">
                <span>Copy</span>
              </button>
              <a class="famous-source-link" href="https://github.com/anthropics/skills/blob/main/skills/xlsx/SKILL.md">View source</a>
            </div>
          </div>
        </td>
      </tr>
    </tbody>
  </table>
</div>

<h3 id="frontend-design">Frontend Design</h3>

Use this when you want the most convincing first run: the prompt is concrete, the output is visible, and the value of a reusable skill is easy to judge from the result.

<h3 id="setup-skillx">Setup skillx</h3>

Use this when you publish a skill and want a tidy "Run with skillx" block in your README. It detects the skill, infers the hosting platform, and inserts a copyable command block with diff preview before writing.

<h3 id="webapp-testing">Webapp Testing</h3>

Use this when you already have a local app running and want a skill to exercise flows, report browser issues, and give you a faster QA loop.

<h3 id="pdf-processing">PDF Processing</h3>

Use this when the task is document-heavy and you need a focused capability right now without turning PDF handling into a permanent part of your setup.

<h3 id="mcp-builder">MCP Builder</h3>

Use this when you need an MCP server faster than you want to write framework boilerplate. It is strong for API-backed agents, tool naming, and workflow-oriented server design.

<h3 id="claude-api">Claude API</h3>

Use this when the problem is not “build a UI” but “ship a better Claude integration.” It is especially useful for prompt caching, streaming responses, tool use, and model-version migrations.

<h3 id="canvas-design">Canvas Design</h3>

Use this when the result should feel like a designed artifact, not just an app screen. It is better suited to visual announcements, posters, hero graphics, and other canvas-first outputs.

<h3 id="docx">DOCX</h3>

Use this when the real deliverable is a Word document with formatting, not just text the model prints back. It is useful for memos, reports, templates, and other document-heavy workflows.

<h3 id="xlsx">XLSX</h3>

Use this when the important artifact is a workbook, not a note. It is a strong fit for cleaning messy sheets, adding formulas, and delivering spreadsheet-ready outputs instead of generic analysis text.

## When to use Famous Skills

Use these when you want a proven workflow that already exists upstream and you do not need to inspect or modify the skill source. If you want to learn the repository's own skill patterns or build from a local template, use the official examples instead.

export const famousSkills = [
  {
    slug: 'frontend-design',
    title: 'Frontend Design',
    runUrl: 'https://github.com/anthropics/skills/tree/main/skills/frontend-design',
    sourceUrl: 'https://github.com/anthropics/skills/blob/main/skills/frontend-design/SKILL.md',
    docsHref: '/getting-started/famous-skills/#frontend-design',
    recommendedReason:
      'Best first impression: it turns a vague brief into a visible interface and shows the quality of a curated skill immediately.',
    bestFor: 'Turning a rough product brief into a visible UI result',
    homepageTitle: 'Redesign a landing page',
    homepageBody:
      'Use a public Agent Skill to redesign a real landing page section with production-ready code.',
    homepagePrompt: 'Redesign the hero section.',
    homepageNote: 'A strong first run if you want a visible, high-signal result.',
    docsDescription:
      'Use this when you want the most convincing first run: the prompt is concrete, the output is visible, and the value of a reusable skill is easy to judge from the result.',
    docsPrompt: 'Design a distinctive landing page for a developer tool',
  },
  {
    slug: 'setup-skillx',
    title: 'Setup skillx',
    runUrl: 'https://github.com/skillx-run/skillx/tree/main/examples/skills/setup-skillx',
    sourceUrl: 'https://github.com/skillx-run/skillx/blob/main/examples/skills/setup-skillx/SKILL.md',
    docsHref: '/getting-started/famous-skills/#setup-skillx',
    recommendedReason:
      'Best when you author skills yourself and want a clean Run-with-skillx block in your README without hand-writing it.',
    bestFor: 'Advertising your own skill so others can try it with skillx',
    homepageTitle: 'Advertise your skill',
    homepageBody:
      'Generate a Run-with-skillx block for your skill\'s README so others can try it in one command.',
    homepagePrompt: '',
    homepageNote: 'The skill is conversational — just run it and it will walk you through detecting your project and drafting the block.',
    docsDescription:
      'Use this when you publish a skill and want a tidy "Run with skillx" block in your README. It detects the skill, infers the hosting platform, and inserts a copyable command block with diff preview before writing. Runs without a free-text prompt — the skill drives the conversation.',
    docsPrompt: '',
  },
  {
    slug: 'webapp-testing',
    title: 'Webapp Testing',
    runUrl: 'https://github.com/anthropics/skills/tree/main/skills/webapp-testing',
    sourceUrl: 'https://github.com/anthropics/skills/blob/main/skills/webapp-testing/SKILL.md',
    docsHref: '/getting-started/famous-skills/#webapp-testing',
    recommendedReason:
      'Best when you already have a running app and want the skill to interact with it directly in the browser.',
    bestFor: 'Browser-driven checks against a running local app',
    homepageTitle: 'Test a local web app',
    homepageBody:
      'Run a testing skill from GitHub to inspect a local app, validate flows, and catch UI issues.',
    homepagePrompt:
      'Test the signup flow on http://localhost:3000 and report any broken states or console errors.',
    homepageNote: 'Good for engineers who want a practical verification workflow.',
    docsDescription:
      'Use this when you already have a local app running and want a skill to exercise flows, report browser issues, and give you a faster QA loop.',
    docsPrompt: 'Test my local web app at http://localhost:3000 for UI regressions and console errors',
  },
  {
    slug: 'pdf-processing',
    title: 'PDF Processing',
    runUrl: 'https://github.com/anthropics/skills/tree/main/skills/pdf',
    sourceUrl: 'https://github.com/anthropics/skills/blob/main/skills/pdf/SKILL.md',
    docsHref: '/getting-started/famous-skills/#pdf-processing',
    recommendedReason:
      'Best for document-heavy work when you need extraction, transformation, or form handling rather than UI or browser automation.',
    bestFor: 'One-off extraction and transformation work for PDF-heavy tasks',
    homepageTitle: 'Process a PDF for one task',
    homepageBody:
      'Pull a PDF skill from GitHub, use it once, and leave nothing permanently installed afterward.',
    homepagePrompt:
      'Extract the tables from ./reports/q1.pdf and summarize the key changes.',
    homepageNote: 'Useful when you need a capability now, not a permanent install.',
    docsDescription:
      'Use this when the task is document-heavy and you need a focused capability right now without turning PDF handling into a permanent part of your setup.',
    docsPrompt: 'Extract the text, tables, and form fields from this PDF',
  },
  {
    slug: 'mcp-builder',
    title: 'MCP Builder',
    runUrl: 'https://github.com/anthropics/skills/tree/main/skills/mcp-builder',
    sourceUrl: 'https://github.com/anthropics/skills/blob/main/skills/mcp-builder/SKILL.md',
    docsHref: '/getting-started/famous-skills/#mcp-builder',
    recommendedReason:
      'Best when you want to turn an external API into a usable MCP server instead of hand-rolling scaffolding and tool definitions.',
    bestFor: 'Scaffolding an MCP server around a real API or service',
    homepageTitle: 'Build an MCP server',
    homepageBody:
      'Generate the structure and workflow for an MCP server that wraps a real external service.',
    homepagePrompt:
      'Create an MCP server plan and starter implementation for a GitHub Issues integration.',
    homepageNote: 'Useful when your next step is tool-building, not app styling.',
    docsDescription:
      'Use this when you need an MCP server faster than you want to write framework boilerplate. It is strong for API-backed agents, tool naming, and workflow-oriented server design.',
    docsPrompt: 'Create an MCP server plan and starter implementation for a GitHub Issues integration',
  },
  {
    slug: 'claude-api',
    title: 'Claude API',
    runUrl: 'https://github.com/anthropics/skills/tree/main/skills/claude-api',
    sourceUrl: 'https://github.com/anthropics/skills/blob/main/skills/claude-api/SKILL.md',
    docsHref: '/getting-started/famous-skills/#claude-api',
    recommendedReason:
      'Best when you are already building against Anthropic APIs and need help with caching, streaming, tool use, or model upgrades.',
    bestFor: 'Building or debugging Anthropic API integrations',
    homepageTitle: 'Ship a Claude API feature',
    homepageBody:
      'Improve an Anthropic integration with better defaults for caching, streaming, and tool use.',
    homepagePrompt:
      'Add prompt caching and streaming to my Anthropic API app and explain the code changes.',
    homepageNote: 'Good for product engineers already wiring LLM features into apps.',
    docsDescription:
      'Use this when the problem is not “build a UI” but “ship a better Claude integration.” It is especially useful for prompt caching, streaming responses, tool use, and model-version migrations.',
    docsPrompt: 'Add prompt caching and streaming to my Anthropic API app and explain the code changes',
  },
  {
    slug: 'canvas-design',
    title: 'Canvas Design',
    runUrl: 'https://github.com/anthropics/skills/tree/main/skills/canvas-design',
    sourceUrl: 'https://github.com/anthropics/skills/blob/main/skills/canvas-design/SKILL.md',
    docsHref: '/getting-started/famous-skills/#canvas-design',
    recommendedReason:
      'Best when you want a more visual, poster-like artifact than a conventional webpage or product screen.',
    bestFor: 'Designing polished visual assets and canvas-style layouts',
    homepageTitle: 'Design a visual artifact',
    homepageBody:
      'Generate a more graphic, editorial output for posters, launch assets, and presentation visuals.',
    homepagePrompt:
      'Create a bold product announcement poster in a modern editorial style.',
    homepageNote: 'A good pick when you want something more graphic than a normal landing page.',
    docsDescription:
      'Use this when the result should feel like a designed artifact, not just an app screen. It is better suited to visual announcements, posters, hero graphics, and other canvas-first outputs.',
    docsPrompt: 'Create a bold product announcement poster in a modern editorial style',
  },
  {
    slug: 'docx',
    title: 'DOCX',
    runUrl: 'https://github.com/anthropics/skills/tree/main/skills/docx',
    sourceUrl: 'https://github.com/anthropics/skills/blob/main/skills/docx/SKILL.md',
    docsHref: '/getting-started/famous-skills/#docx',
    recommendedReason:
      'Best when the deliverable is a polished Word document rather than plain text or Markdown.',
    bestFor: 'Creating or editing polished .docx deliverables',
    homepageTitle: 'Generate a Word doc',
    homepageBody:
      'Create or reshape a professional .docx deliverable instead of stopping at plain text.',
    homepagePrompt:
      'Create a polished project update memo as a .docx file with headings and a summary table.',
    homepageNote: 'Useful when the output format matters as much as the content.',
    docsDescription:
      'Use this when the real deliverable is a Word document with formatting, not just text the model prints back. It is useful for memos, reports, templates, and other document-heavy workflows.',
    docsPrompt: 'Create a polished project update memo as a .docx file with headings and a summary table',
  },
  {
    slug: 'xlsx',
    title: 'XLSX',
    runUrl: 'https://github.com/anthropics/skills/tree/main/skills/xlsx',
    sourceUrl: 'https://github.com/anthropics/skills/blob/main/skills/xlsx/SKILL.md',
    docsHref: '/getting-started/famous-skills/#xlsx',
    recommendedReason:
      'Best when the task is trapped in a spreadsheet and you need cleanup, formulas, or restructuring instead of prose.',
    bestFor: 'Cleaning, analyzing, and restructuring spreadsheet workbooks',
    homepageTitle: 'Fix a spreadsheet workflow',
    homepageBody:
      'Open, clean, and improve spreadsheet-heavy work without turning it into a manual Excel session.',
    homepagePrompt:
      'Clean this sales workbook, add summary formulas, and prepare it for review.',
    homepageNote: 'Strong for data-heavy work where the final deliverable is still a spreadsheet.',
    docsDescription:
      'Use this when the important artifact is a workbook, not a note. It is a strong fit for cleaning messy sheets, adding formulas, and delivering spreadsheet-ready outputs instead of generic analysis text.',
    docsPrompt: 'Clean this sales workbook, add summary formulas, and prepare it for review',
  },
];

export const primaryFamousSkill = famousSkills[0];

export function getFamousSkill(slug) {
  return famousSkills.find((skill) => skill.slug === slug);
}

export function buildSkillxRunCommand(skill, prompt) {
  if (!prompt) return `skillx run ${skill.runUrl}`;
  return `skillx run ${skill.runUrl} "${prompt}"`;
}

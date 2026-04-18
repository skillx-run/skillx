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
      'Use a public GitHub skill to redesign a real landing page section with production-ready code.',
    homepagePrompt:
      'Redesign the hero section of this landing page for higher conversion. Keep the existing stack and return production-ready code.',
    homepageNote: 'A strong first run if you want a visible, high-signal result.',
    docsDescription:
      'Use this when you want the most convincing first run: the prompt is concrete, the output is visible, and the value of a reusable skill is easy to judge from the result.',
    docsPrompt: 'Design a distinctive landing page for a developer tool',
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
];

export const primaryFamousSkill = famousSkills[0];

export function getFamousSkill(slug) {
  return famousSkills.find((skill) => skill.slug === slug);
}

export function buildSkillxRunCommand(skill, prompt) {
  return `skillx run ${skill.runUrl} "${prompt}"`;
}

import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { primaryFamousSkill } from '../src/data/famous-skills.mjs';

const html = readFileSync(resolve('dist/index.html'), 'utf8');

const checks = [
  'og-image.png',
  'twitter:image',
  'data-home-section="hero"',
  'Run a GitHub skill in one command.',
  'Paste a public GitHub skill URL.',
  'Run your first skill',
  'data-home-section="value-split"',
  'Trying a GitHub skill should not feel like setup work.',
  'Manual setup',
  'Run with skillx',
  'data-home-section="use-cases"',
  'Start with a real skill, not a toy example.',
  'Redesign a landing page',
  'Test a local web app',
  'Process a PDF for one task',
  'data-home-section="mechanism-trust"',
  'Trust comes from the workflow, not a promise.',
  'Built-in analyzers',
  'Risk-based gate',
  'data-home-section="final-cta"',
  'Install skillx, then run your first GitHub skill.',
  'curl -fsSL https://skillx.run/install.sh | sh',
  primaryFamousSkill.runUrl,
  'https://github.com/skillx-run/skillx',
];

let cursor = 0;

for (const check of checks) {
  const index = html.indexOf(check, cursor);
  if (index === -1) {
    throw new Error(`Missing homepage fragment: ${check}`);
  }
  cursor = index + check.length;
}

console.log(`Homepage smoke checks passed (${checks.length} checks).`);

import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { buildSkillxRunCommand, getFamousSkill, primaryFamousSkill } from '../src/data/famous-skills.mjs';

const authorSkill = getFamousSkill('setup-skillx');
const authorCommand = buildSkillxRunCommand(authorSkill, authorSkill.homepagePrompt);

const html = readFileSync(resolve('dist/index.html'), 'utf8');

const checks = [
  'og-image.png',
  'twitter:image',
  'data-home-section="hero"',
  'Run an Agent Skill in one command.',
  'Paste a public Agent Skill URL.',
  'Run your first skill',
  'data-home-section="value-split"',
  'Most skill managers install. skillx runs.',
  'Install it into your agent',
  '4 steps per skill',
  'skillx does the rest',
  '1 command',
  'data-home-section="use-cases"',
  'Start with a real skill, not a toy example.',
  'Redesign a landing page',
  'Advertise your skill',
  'Test a local web app',
  'data-home-section="mechanism-trust"',
  'Trust comes from the workflow, not a promise.',
  'data-home-section="final-cta"',
  'Install skillx, then run your first Agent Skill.',
  'curl -fsSL https://skillx.run/install.sh | sh',
  primaryFamousSkill.runUrl,
  primaryFamousSkill.homepagePrompt,
  'For skill authors',
  'Shipping a skill? Generate a Run-with-skillx block for your README in one command.',
  authorCommand,
  '/guides/advertise-your-skill/',
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

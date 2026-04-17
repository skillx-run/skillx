import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

const html = readFileSync(resolve('dist/index.html'), 'utf8');

const checks = [
  'data-home-section="hero"',
  'Run any agent skill in one command.',
  'Make skills easier to try, easier to trust, and easier to adopt.',
  'Install skillx',
  'data-home-section="value-split"',
  'For Skill Users',
  'Skip manual install and cleanup',
  'For Skill Developers',
  'Share a link and reduce adoption friction',
  'data-home-section="mechanism-trust"',
  'resolve',
  'clean',
  '30 rules',
  '32+ agents',
  '18+ sources',
  '5 risk levels',
  'scan before inject',
];

let cursor = 0;

for (const check of checks) {
  const index = html.indexOf(check, cursor);
  if (index === -1) {
    throw new Error(`Missing homepage fragment: ${check}`);
  }
  cursor = index + check.length;
}

console.log('Homepage smoke checks passed (4 checks).');

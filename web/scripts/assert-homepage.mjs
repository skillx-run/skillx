import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

const html = readFileSync(resolve('dist/index.html'), 'utf8');

const checks = [
  'data-home-section="hero"',
  'Run any agent skill in one command.',
  'Make skills easier to try, easier to trust, and easier to adopt.',
  'Install skillx',
  '32+ agents',
  '18+ sources',
  '30 rules',
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

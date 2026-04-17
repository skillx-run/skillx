import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

const html = readFileSync(resolve('dist/index.html'), 'utf8');

const checks = [
  'data-home-section="hero"',
  'data-home-section="value-split"',
  'data-home-section="mechanism-trust"',
  'data-home-section="final-cta"',
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

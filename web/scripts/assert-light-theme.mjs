import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

const pages = ['dist/getting-started/index.html', 'dist/cli/run/index.html'];

const mustInclude = ["dataset.theme = 'light'"];

const mustNotInclude = [
  'StarlightThemeProvider',
  "prefers-color-scheme: light",
];

for (const page of pages) {
  const html = readFileSync(resolve(page), 'utf8');

  for (const fragment of mustInclude) {
    if (!html.includes(fragment)) {
      throw new Error(`${page}: missing fragment ${JSON.stringify(fragment)}`);
    }
  }

  for (const fragment of mustNotInclude) {
    if (html.includes(fragment)) {
      throw new Error(
        `${page}: found forbidden fragment ${JSON.stringify(fragment)} — ThemeProvider override may be broken`,
      );
    }
  }
}

console.log(`Light-theme checks passed (${pages.length} pages).`);

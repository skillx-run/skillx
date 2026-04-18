import { execFileSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

const chromeBinary = '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome';
const input = resolve('scripts/og-image.html');
const output = resolve('public/og-image.png');

if (!existsSync(chromeBinary)) {
  throw new Error(`Chrome binary not found: ${chromeBinary}`);
}

const url = pathToFileURL(input).href;

execFileSync(
  chromeBinary,
  [
    '--headless=new',
    '--disable-gpu',
    '--hide-scrollbars',
    '--run-all-compositor-stages-before-draw',
    '--virtual-time-budget=5000',
    '--window-size=1200,630',
    `--screenshot=${output}`,
    url,
  ],
  { stdio: 'inherit' },
);

console.log(`Rendered og image to ${output}`);

import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import {
  buildSkillxRunCommand,
  famousSkills,
  primaryFamousSkill,
} from '../src/data/famous-skills.mjs';

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

function assertUrlOk(url) {
  const status = execFileSync(
    'curl',
    ['-I', '-L', '-sS', '--max-time', '20', '-o', '/dev/null', '-w', '%{http_code}', url],
    { encoding: 'utf8' },
  ).trim();

  assert(/^2|3/.test(status), `URL check failed (${status}): ${url}`);
}

const slugs = new Set();
const docsHrefs = new Set();
const runUrls = new Set();

assert(famousSkills.length >= 9, 'Famous skills list must contain at least 9 curated skills');

for (const skill of famousSkills) {
  assert(!slugs.has(skill.slug), `Duplicate famous skill slug: ${skill.slug}`);
  assert(!docsHrefs.has(skill.docsHref), `Duplicate docsHref: ${skill.docsHref}`);
  assert(!runUrls.has(skill.runUrl), `Duplicate runUrl: ${skill.runUrl}`);
  slugs.add(skill.slug);
  docsHrefs.add(skill.docsHref);
  runUrls.add(skill.runUrl);
}

assert(primaryFamousSkill.slug === famousSkills[0].slug, 'Primary famous skill must be first');

const homepageComponent = readFileSync(resolve('src/components/home/home-use-cases.astro'), 'utf8');
assert(
  homepageComponent.includes('famousSkills') &&
    homepageComponent.includes('slice(0, 3)') &&
    homepageComponent.includes("../../data/famous-skills.mjs"),
  'Homepage use cases must consume the shared famous skills data source',
);

const firstRunDoc = readFileSync(resolve('src/content/docs/getting-started/first-run.md'), 'utf8');
assert(
  firstRunDoc.includes(primaryFamousSkill.runUrl) &&
    firstRunDoc.includes(primaryFamousSkill.homepagePrompt),
  'First Run doc must include the primary famous skill command',
);

const famousSkillsDoc = readFileSync(
  resolve('src/content/docs/getting-started/famous-skills.md'),
  'utf8',
);
assert(
  famousSkillsDoc.includes('famous-copy-button'),
  'Famous Skills doc must include explicit copy-button UI hooks',
);
for (const skill of famousSkills) {
  assert(
    famousSkillsDoc.includes(skill.runUrl) &&
      famousSkillsDoc.includes(skill.sourceUrl) &&
      famousSkillsDoc.includes(`id="${skill.slug}"`),
    `Famous Skills doc must include URL, source, and anchor for ${skill.slug}`,
  );
}

const command = buildSkillxRunCommand(primaryFamousSkill, primaryFamousSkill.homepagePrompt);
assert(
  command.includes(primaryFamousSkill.runUrl),
  'Primary command should include the primary famous skill URL',
);

for (const skill of famousSkills) {
  assertUrlOk(skill.runUrl);
  assertUrlOk(skill.sourceUrl);
}

console.log(`Famous skills checks passed (${famousSkills.length} skills validated).`);

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
    homepageComponent.includes("../../data/famous-skills.mjs"),
  'Homepage use cases must consume the shared famous skills data source',
);

const firstRunDoc = readFileSync(resolve('src/content/docs/getting-started/first-run.md'), 'utf8');
assert(
  /<SkillCommand[\s\S]*slug="frontend-design"/.test(firstRunDoc),
  'First Run doc must use the shared SkillCommand component',
);

const famousSkillsDoc = readFileSync(
  resolve('src/content/docs/getting-started/famous-skills.md'),
  'utf8',
);
assert(
  /<FamousSkillsTable\s*\/>/.test(famousSkillsDoc),
  'Famous Skills doc must render from the shared FamousSkillsTable component',
);

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

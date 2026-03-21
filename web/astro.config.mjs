import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

export default defineConfig({
  site: 'https://skillx.run',
  integrations: [
    starlight({
      title: 'skillx',
      description: 'npx for Agent Skills — fetch, scan, inject, run, clean in one command',
      social: [
        { icon: 'github', label: 'GitHub', href: 'https://github.com/anthropics/skillx' },
      ],
      sidebar: [
        {
          label: 'Getting Started',
          items: [
            { label: 'Introduction', slug: 'getting-started' },
            { label: 'Installation', slug: 'getting-started/installation' },
            { label: 'First Run', slug: 'getting-started/first-run' },
          ],
        },
        {
          label: 'CLI Reference',
          items: [
            { label: 'run', slug: 'cli/run' },
            { label: 'install', slug: 'cli/install' },
            { label: 'uninstall', slug: 'cli/uninstall' },
            { label: 'list', slug: 'cli/list' },
            { label: 'update', slug: 'cli/update' },
            { label: 'init', slug: 'cli/init' },
            { label: 'scan', slug: 'cli/scan' },
            { label: 'agents', slug: 'cli/agents' },
            { label: 'info', slug: 'cli/info' },
            { label: 'config', slug: 'cli/config' },
          ],
        },
        {
          label: 'Platforms',
          items: [
            { label: 'Overview', slug: 'platforms/overview' },
            { label: 'Git Hosts', slug: 'platforms/git-hosts' },
            { label: 'Skill Directories', slug: 'platforms/skill-directories' },
          ],
        },
        {
          label: 'Agents',
          items: [
            { label: 'Overview', slug: 'agents/overview' },
            { label: 'CLI Agents', slug: 'agents/cli-agents' },
            { label: 'IDE Agents', slug: 'agents/ide-agents' },
            { label: 'Universal', slug: 'agents/universal' },
          ],
        },
        {
          label: 'Security',
          items: [
            { label: 'Overview', slug: 'security/overview' },
            { label: 'Risk Levels', slug: 'security/risk-levels' },
            { label: 'Rules', slug: 'security/rules' },
          ],
        },
        {
          label: 'Guides',
          items: [
            { label: 'Writing Skills', slug: 'guides/writing-skills' },
            { label: 'Agent Adapters', slug: 'guides/agent-adapters' },
            { label: 'CI Integration', slug: 'guides/ci-integration' },
          ],
        },
        {
          label: 'Reference',
          items: [
            { label: 'CLI Flags', slug: 'reference/cli-flags' },
            { label: 'config.toml', slug: 'reference/config-toml' },
            { label: 'Manifest', slug: 'reference/manifest' },
          ],
        },
      ],
      customCss: ['./src/styles/custom.css'],
    }),
  ],
});

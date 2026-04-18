import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

export default defineConfig({
  site: 'https://skillx.run',
  integrations: [
    starlight({
      title: 'skillx',
      description: 'npx for Agent Skills — fetch, scan, inject, run, clean in one command',
      components: {
        ThemeSelect: './src/components/empty.astro',
        ThemeProvider: './src/components/empty.astro',
      },
      head: [
        {
          tag: 'script',
          content: `document.documentElement.dataset.theme = 'light';`,
        },
      ],
      social: [
        { icon: 'github', label: 'GitHub', href: 'https://github.com/skillx-run/skillx' },
        { icon: 'x.com', label: 'X', href: 'https://x.com/SkillxRun' },
      ],
      sidebar: [
        {
          label: 'Run a GitHub Skill',
          items: [
            { label: 'Introduction', slug: 'getting-started' },
            { label: 'Installation', slug: 'getting-started/installation' },
            { label: 'First Run', slug: 'getting-started/first-run' },
            { label: 'FAQ', slug: 'getting-started/faq' },
            { label: 'Troubleshooting', slug: 'getting-started/troubleshooting' },
            { label: 'Famous Skills', slug: 'getting-started/famous-skills' },
            { label: 'Official Examples', slug: 'examples/overview' },
          ],
        },
        {
          label: 'Use skillx in Real Work',
          items: [
            { label: 'Run Skills', slug: 'cli/run' },
            { label: 'Scan Skills', slug: 'cli/scan' },
            { label: 'Manage Project Skills', slug: 'guides/manage-project-skills' },
            { label: 'Agents', slug: 'agents/overview' },
          ],
        },
        {
          label: 'Trust & Security',
          items: [
            { label: 'Security Overview', slug: 'security/overview' },
            { label: 'Risk Levels', slug: 'security/risk-levels' },
            { label: 'Rules', slug: 'security/rules' },
            { label: 'CI Integration', slug: 'guides/ci-integration' },
          ],
        },
        {
          label: 'Build Skills',
          items: [
            { label: 'Writing Skills', slug: 'guides/writing-skills' },
            { label: 'Advertise Your Skill', slug: 'guides/advertise-your-skill' },
            { label: 'Official Example Patterns', slug: 'examples/overview' },
            { label: 'Agent Adapters', slug: 'guides/agent-adapters' },
          ],
        },
        {
          label: 'Reference',
          items: [
            { label: 'CLI Flags', slug: 'reference/cli-flags' },
            { label: 'config.toml', slug: 'reference/config-toml' },
            { label: 'Manifest', slug: 'reference/manifest' },
            { label: 'Git Hosts', slug: 'platforms/git-hosts' },
            { label: 'Source URLs', slug: 'platforms/overview' },
          ],
        },
      ],
      customCss: ['./src/styles/custom.css'],
    }),
  ],
});

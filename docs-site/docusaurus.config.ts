import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';
import gruvboxTheme from './src/css/prism-gruvbox';

const config: Config = {
  title: 'Prisma AIRS Harness',
  tagline: 'A local terminal agent connected through Prisma AIRS AI Gateway',
  favicon: 'img/logo.svg',
  url: 'https://cdot65.github.io',
  baseUrl: '/prisma-airs-harness/',
  organizationName: 'cdot65',
  projectName: 'prisma-airs-harness',
  future: {v4: true},
  trailingSlash: true,
  onBrokenLinks: 'throw',
  onBrokenAnchors: 'throw',
  markdown: {format: 'detect', mermaid: true, hooks: {onBrokenMarkdownLinks: 'throw'}},
  themes: ['@docusaurus/theme-mermaid'],
  i18n: {defaultLocale: 'en', locales: ['en']},
  presets: [['classic', {
    docs: {sidebarPath: './sidebars.ts', routeBasePath: '/'},
    blog: false,
    theme: {customCss: './src/css/custom.css'},
  } satisfies Preset.Options]],
  themeConfig: {
    docs: {sidebar: {hideable: true}},
    colorMode: {defaultMode: 'dark', disableSwitch: true, respectPrefersColorScheme: false},
    navbar: {
      title: 'Prisma AIRS Harness',
      logo: {alt: 'Prisma AIRS Harness', src: 'img/logo.svg'},
      items: [
        {type: 'docSidebar', sidebarId: 'docs', label: 'Docs', position: 'left'},
        {type: 'docSidebar', sidebarId: 'commands', label: 'CLI Reference', position: 'left'},
        {type: 'docSidebar', sidebarId: 'developers', label: 'Developers', position: 'left'},
        {href: 'https://github.com/cdot65/prisma-airs-harness', label: 'GitHub', position: 'right'},
      ],
    },
    footer: {
      style: 'dark',
      links: [
        {title: 'Harness', items: [
          {label: 'Getting Started', to: '/getting-started/'},
          {label: 'CLI Reference', to: '/reference/airs/'},
          {label: 'Releases', to: '/guides/releases/'},
        ]},
        {title: 'Prisma AIRS', items: [
          {label: 'CLI', href: 'https://cdot65.github.io/prisma-airs-cli/'},
          {label: 'SDK', href: 'https://cdot65.github.io/prisma-airs-sdk/'},
          {label: 'Reference Architecture', href: 'https://cdot65.github.io/prisma-airs-reference-architecture/'},
        ]},
        {title: 'Source', items: [
          {label: 'Forgejo · issues and contributions', href: 'https://git.cdot.io/cdot/prisma-airs-harness'},
          {label: 'GitHub mirror', href: 'https://github.com/cdot65/prisma-airs-harness'},
        ]},
      ],
      copyright: `Copyright © ${new Date().getFullYear()} cdot65. Apache-2.0. Built with Docusaurus.`,
    },
    prism: {theme: gruvboxTheme, darkTheme: gruvboxTheme,
      additionalLanguages: ['bash', 'json', 'yaml', 'python', 'powershell', 'toml', 'diff', 'rust']},
  } satisfies Preset.ThemeConfig,
};
export default config;

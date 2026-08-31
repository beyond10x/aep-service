import type {Config} from '@docusaurus/types';
import type {Options, ThemeConfig} from '@docusaurus/preset-classic';

const config: Config = {
  title: 'AEP Service',
  tagline: 'The governed authority for engineering decisions',
  favicon: 'img/favicon.svg',
  url: 'https://beyond10x.github.io',
  baseUrl: '/aep-service/',
  organizationName: 'beyond10x',
  projectName: 'aep-service',
  onBrokenLinks: 'throw',
  markdown: {hooks: {onBrokenMarkdownLinks: 'throw'}},
  presets: [
    [
      'classic',
      {
        docs: {sidebarPath: './sidebars.ts', routeBasePath: 'docs'},
        blog: false,
        theme: {customCss: './src/css/custom.css'},
      } satisfies Options,
    ],
  ],
  themeConfig: {
    colorMode: {defaultMode: 'dark', respectPrefersColorScheme: true},
    navbar: {
      title: 'AEP Service',
      items: [
        {to: '/docs/intro', label: 'Docs', position: 'left'},
        {to: '/api', label: 'API', position: 'left'},
        {href: 'https://github.com/beyond10x/aep-service', label: 'GitHub', position: 'right'},
      ],
    },
    footer: {
      style: 'dark',
      links: [
        {title: 'Learn', items: [{label: 'Quickstart', to: '/docs/quickstart'}, {label: 'Security', to: '/docs/security'}]},
        {title: 'Source', items: [{label: 'Repository', href: 'https://github.com/beyond10x/aep-service'}, {label: 'Engineering Protocols', href: 'https://github.com/beyond10x/engineering-protocols'}, {label: 'Entity Runtime', href: 'https://github.com/beyond10x/entity-runtime'}]},
      ],
      copyright: `Copyright © ${new Date().getFullYear()} beyond10x. Apache-2.0.`,
    },
  } satisfies ThemeConfig,
};

export default config;

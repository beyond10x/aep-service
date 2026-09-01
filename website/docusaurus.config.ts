import type {Config} from '@docusaurus/types';
import type {Options, ThemeConfig} from '@docusaurus/preset-classic';
import docsSystemPlugin from '@beyond10x/docs-system/docusaurus';

const config: Config = {
  title: 'AEP Service',
  tagline: 'The governed authority for engineering decisions',
  favicon: 'img/favicon.svg',
  future: {v4: true},
  url: 'https://beyond10x.github.io',
  baseUrl: '/aep-service/',
  organizationName: 'beyond10x',
  projectName: 'aep-service',
  trailingSlash: false,
  onBrokenLinks: 'throw',
  plugins: [docsSystemPlugin],
  onBrokenAnchors: 'throw',
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
    image: 'img/social-card.svg',
    metadata: [
      {
        name: 'keywords',
        content: 'AEP Service, engineering protocols, governed engineering, entity runtime, agent planning, attributable decisions, semantic API',
      },
    ],
    colorMode: {defaultMode: 'dark', respectPrefersColorScheme: true},
    navbar: {
      title: 'AEP Service',
      hideOnScroll: true,
      logo: {alt: 'AEP Service linked authority mark', src: 'img/mark.svg'},
      items: [
        {href: 'https://beyond10x.github.io/getting-started/', label: 'beyond10x', position: 'left'},
        {href: 'https://beyond10x.github.io/getting-started/ecosystem', label: 'Ecosystem', position: 'left'},
        {to: '/docs/intro', label: 'Docs', position: 'left'},
        {to: '/docs/architecture', label: 'Architecture', position: 'left'},
        {to: '/api', label: 'API', position: 'left'},
        {href: 'https://github.com/beyond10x/aep-service', label: 'GitHub', position: 'right'},
      ],
    },
    footer: {
      style: 'dark',
      links: [
        {title: 'Evaluate', items: [{label: 'Run the preview', to: '/docs/quickstart'}, {label: 'Architecture', to: '/docs/architecture'}, {label: 'API reference', to: '/api'}]},
        {title: 'Operate', items: [{label: 'Configuration', to: '/docs/configuration'}, {label: 'Reliability', to: '/docs/reliability'}, {label: 'Security', to: '/docs/security'}]},
        {title: 'Ecosystem', items: [{label: 'AEP', href: 'https://beyond10x.github.io/aep/'}, {label: 'ESS', href: 'https://beyond10x.github.io/ess/'}, {label: 'Entity Runtime', href: 'https://github.com/beyond10x/entity-runtime'}, {label: 'Source on GitHub', href: 'https://github.com/beyond10x/aep-service'}]},
      ],
      copyright: `Copyright © ${new Date().getFullYear()} beyond10x. Apache-2.0.`,
    },
  } satisfies ThemeConfig,
};

export default config;

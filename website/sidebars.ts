import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

const sidebars: SidebarsConfig = {
  docs: [
    {
      type: 'category',
      label: 'Start',
      collapsed: false,
      items: ['intro', 'quickstart'],
    },
    {
      type: 'category',
      label: 'Understand',
      collapsed: false,
      items: ['architecture', 'concepts'],
    },
    {
      type: 'category',
      label: 'Integrate',
      collapsed: false,
      items: ['commands-and-queries', 'http-contract', 'reliability'],
    },
    {
      type: 'category',
      label: 'Operate',
      collapsed: false,
      items: ['configuration', 'operations', 'security', 'release-status'],
    },
  ],
};

export default sidebars;

import type { BaseLayoutProps } from 'fumadocs-ui/layouts/shared';

export const gitConfig = {
  user: 'neel',
  repo: 'Qryvanta',
  branch: 'main',
};

export function baseOptions(): BaseLayoutProps {
  return {
    themeSwitch: {
      enabled: false,
    },
    nav: {
      title: 'Qryvanta Docs',
    },
    githubUrl: `https://github.com/${gitConfig.user}/${gitConfig.repo}`,
    links: [
      {
        text: 'Quickstart',
        url: '/docs/quickstart',
        active: 'nested-url',
      },
      {
        text: 'Workspace',
        url: '/docs/workspace',
        active: 'nested-url',
      },
      {
        text: 'Concepts',
        url: '/docs/concepts',
        active: 'nested-url',
      },
      {
        text: 'Operations',
        url: '/docs/operations',
        active: 'nested-url',
      },
    ],
  };
}

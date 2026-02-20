import type { BaseLayoutProps } from 'fumadocs-ui/layouts/shared';

export const gitConfig = {
  user: 'neel',
  repo: 'Qryvanta',
  branch: 'main',
};

export function baseOptions(): BaseLayoutProps {
  return {
    nav: {
      title: 'Qryvanta Docs',
    },
    githubUrl: `https://github.com/${gitConfig.user}/${gitConfig.repo}`,
    links: [
      {
        text: 'Platform',
        url: '/docs/platform/metadata-runtime',
        active: 'nested-url',
      },
      {
        text: 'Architecture',
        url: '/docs/architecture/overview',
        active: 'nested-url',
      },
      {
        text: 'Operations',
        url: '/docs/operations/self-hosting',
        active: 'nested-url',
      },
    ],
  };
}

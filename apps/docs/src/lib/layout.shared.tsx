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
      title: (
        <span className="docs-brand">
          <span className="docs-brand-mark" aria-hidden="true">
            Q
          </span>
          <span className="docs-brand-copy">
            <span className="docs-brand-name">Qryvanta Docs</span>
            <span className="docs-brand-tag">OSS platform manual</span>
          </span>
        </span>
      ),
    },
    githubUrl: `https://github.com/${gitConfig.user}/${gitConfig.repo}`,
    links: [
      {
        text: 'Start Here',
        url: '/docs',
        active: 'url',
      },
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

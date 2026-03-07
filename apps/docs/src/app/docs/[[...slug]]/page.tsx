import Link from 'next/link';
import { getPageImage, source } from '@/lib/source';
import { DocsBody, DocsDescription, DocsPage, DocsTitle } from 'fumadocs-ui/layouts/docs/page';
import { notFound } from 'next/navigation';
import { getMDXComponents } from '@/mdx-components';
import type { Metadata } from 'next';
import { createRelativeLink } from 'fumadocs-ui/mdx';
import { LLMCopyButton, ViewOptions } from '@/components/ai/page-actions';
import { gitConfig } from '@/lib/layout.shared';
import { findNeighbour } from 'fumadocs-core/page-tree';

const sectionLabels: Record<string, string> = {
  quickstart: 'Quickstart',
  workspace: 'Workspace',
  concepts: 'Concepts',
  operations: 'Operations',
};

const sectionIndexLinks: Record<string, { href: string; label: string }> = {
  quickstart: { href: '/docs/quickstart', label: 'Quickstart overview' },
  workspace: { href: '/docs/workspace', label: 'Workspace overview' },
  concepts: { href: '/docs/concepts', label: 'Concepts overview' },
  operations: { href: '/docs/operations', label: 'Operations overview' },
};

const sectionDescriptions: Record<string, string> = {
  quickstart: 'Get from local setup to a working first publish.',
  workspace: 'Understand the admin, maker, and worker surfaces.',
  concepts: 'Read the architecture, publish model, and runtime rules.',
  operations: 'Use the self-hosting, security, and reliability runbooks.',
};

export default async function Page(props: PageProps<'/docs/[[...slug]]'>) {
  const params = await props.params;
  const page = source.getPage(params.slug);
  if (!page) notFound();

  const MDX = page.data.body;
  const sectionLabel = page.slugs[0] ? sectionLabels[page.slugs[0]] ?? 'Documentation' : 'Documentation';
  const pagePath = page.slugs.length > 0 ? `/docs/${page.slugs.join('/')}` : '/docs';
  const neighbours = findNeighbour(source.getPageTree(), page.url);
  const sectionLink = page.slugs[0] ? sectionIndexLinks[page.slugs[0]] : undefined;
  const sectionDescription = page.slugs[0]
    ? sectionDescriptions[page.slugs[0]] ?? 'Use the section overview to understand the whole path.'
    : 'Start with the overview, then move into the section that matches your task.';

  return (
    <DocsPage
      toc={page.data.toc}
      full={page.data.full}
      className="docs-page-shell"
      tableOfContent={{
        header: (
          <div className="docs-toc-panel">
            <p className="docs-toc-kicker">{sectionLabel}</p>
            <p className="docs-toc-copy">{sectionDescription}</p>
            {sectionLink ? (
              <Link className="docs-toc-link" href={sectionLink.href}>
                {sectionLink.label}
              </Link>
            ) : null}
          </div>
        ),
        footer: neighbours.next ? (
          <div className="docs-toc-panel docs-toc-panel-quiet">
            <p className="docs-toc-kicker">Up next</p>
            <Link className="docs-toc-link" href={neighbours.next.url}>
              {neighbours.next.name}
            </Link>
          </div>
        ) : undefined,
      }}
      footer={{
        items: neighbours,
        className: 'docs-page-footer',
        children: (
          <div className="docs-page-footer-note">
            <p className="docs-page-footer-label">Need the whole picture?</p>
            <div className="docs-page-footer-links">
              {sectionLink ? <Link href={sectionLink.href}>{sectionLink.label}</Link> : null}
              <Link href="/docs">Start Here</Link>
            </div>
          </div>
        ),
      }}
    >
      <div className="docs-page-header">
        <div className="docs-page-kicker">
          <span className="docs-page-badge">{sectionLabel}</span>
          <span className="docs-page-slug">{pagePath}</span>
        </div>
        {sectionLink ? (
          <Link className="docs-page-section-link" href={sectionLink.href}>
            {sectionLink.label}
          </Link>
        ) : null}
        <DocsTitle className="docs-page-title">{page.data.title}</DocsTitle>
        <DocsDescription className="docs-page-description mb-0">{page.data.description}</DocsDescription>
      </div>
      <div className="docs-page-actions mt-5 flex flex-row items-center gap-2">
        <LLMCopyButton markdownUrl={`${page.url}.mdx`} />
        <ViewOptions
          markdownUrl={`${page.url}.mdx`}
          githubUrl={`https://github.com/${gitConfig.user}/${gitConfig.repo}/blob/${gitConfig.branch}/apps/docs/content/docs/${page.path}`}
        />
      </div>
      <DocsBody className="docs-page-body">
        <MDX
          components={getMDXComponents({
            // this allows you to link to other pages with relative file paths
            a: createRelativeLink(source, page),
          })}
        />
      </DocsBody>
    </DocsPage>
  );
}

export async function generateStaticParams() {
  return source.generateParams();
}

export async function generateMetadata(props: PageProps<'/docs/[[...slug]]'>): Promise<Metadata> {
  const params = await props.params;
  const page = source.getPage(params.slug);
  if (!page) notFound();

  return {
    title: page.data.title,
    description: page.data.description,
    openGraph: {
      images: getPageImage(page).url,
    },
  };
}

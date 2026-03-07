import Link from 'next/link';
import { source } from '@/lib/source';
import { DocsLayout } from 'fumadocs-ui/layouts/docs';
import { baseOptions } from '@/lib/layout.shared';

export default function Layout({ children }: LayoutProps<'/docs'>) {
  return (
    <DocsLayout
      tree={source.getPageTree()}
      sidebar={{
        collapsible: false,
        className: 'docs-sidebar border-e border-fd-border/70 bg-white/72 backdrop-blur',
        banner: (
          <div className="docs-sidebar-panel mx-3 mt-3 rounded-[1.35rem] p-4">
            <p className="docs-sidebar-kicker">Open source docs</p>
            <h2 className="mt-2 text-base font-semibold tracking-tight text-fd-foreground">
              Read Qryvanta like a product, not a code dump
            </h2>
            <p className="mt-2 text-sm leading-6 text-fd-muted-foreground">
              Start with the platform shape, then move through product surfaces, publish rules, and operator runbooks.
            </p>
            <div className="mt-4 grid gap-2">
              <Link className="docs-sidebar-quicklink" href="/docs/quickstart">
                Quickstart
              </Link>
              <Link className="docs-sidebar-quicklink" href="/docs/concepts/platform-architecture">
                Platform Architecture
              </Link>
              <Link className="docs-sidebar-quicklink" href="/docs/operations/self-hosting">
                Self-Hosting
              </Link>
            </div>
          </div>
        ),
        footer: (
          <div className="docs-sidebar-panel mx-3 mb-3 rounded-[1.2rem] p-3 text-xs leading-5 text-fd-muted-foreground">
            <p className="docs-sidebar-kicker">Operator note</p>
            <p className="mt-2">
              Qryvanta is in active development. Validate behavior against the running build before rollout.
            </p>
            <div className="mt-3 flex flex-wrap gap-2">
              <Link className="docs-mini-pill" href="/docs/operations/security-hardening">
                Security
              </Link>
              <Link className="docs-mini-pill" href="/docs/operations/observability">
                Observability
              </Link>
            </div>
          </div>
        ),
      }}
      containerProps={{
        className: 'docs-shell w-full max-w-none',
        style: {
          gridTemplate: `"sidebar sidebar header toc toc"
            "sidebar sidebar toc-popover toc toc"
            "sidebar sidebar main toc toc" 1fr / 0px var(--fd-sidebar-col) minmax(0, calc(var(--fd-layout-width, 97rem) - var(--fd-sidebar-width) - var(--fd-toc-width))) var(--fd-toc-width) minmax(0, 1fr)`,
        },
      }}
      {...baseOptions()}
    >
      {children}
    </DocsLayout>
  );
}

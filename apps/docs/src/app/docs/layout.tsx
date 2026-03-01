import { source } from '@/lib/source';
import { DocsLayout } from 'fumadocs-ui/layouts/docs';
import { baseOptions } from '@/lib/layout.shared';

export default function Layout({ children }: LayoutProps<'/docs'>) {
  return (
    <DocsLayout
      tree={source.getPageTree()}
      sidebar={{
        collapsible: false,
        className: 'border-e border-fd-border/70 bg-white/70 backdrop-blur',
        banner: (
          <div className="mx-3 mt-3 rounded-xl border border-fd-border bg-fd-muted/50 p-3">
            <p className="text-xs font-semibold uppercase tracking-[0.12em] text-fd-muted-foreground">User Docs</p>
            <p className="mt-1 text-sm text-fd-foreground">Guides for admins, makers, worker users, and operators.</p>
          </div>
        ),
        footer: (
          <div className="mx-3 mb-3 rounded-xl border border-fd-border bg-white/80 p-3 text-xs text-fd-muted-foreground">
            Open source docs for `qryvanta.org`.
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

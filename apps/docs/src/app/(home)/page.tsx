import Link from 'next/link';

export default function HomePage() {
  return (
    <div className="mx-auto flex w-full max-w-6xl flex-1 flex-col gap-8 px-6 pb-20 pt-14 md:gap-12 md:px-10 md:pt-20">
      <div className="mx-auto flex max-w-3xl flex-col items-center gap-5 text-center">
        <p className="docs-chip rounded-full px-3 py-1 text-xs font-semibold uppercase tracking-[0.18em]">
          Open Source and Self-Hosting
        </p>
        <h1 className="text-balance text-4xl font-semibold tracking-tight text-fd-foreground md:text-5xl">
          Qryvanta Documentation
        </h1>
        <p className="max-w-2xl text-pretty text-lg text-fd-muted-foreground">
          Practical guides for admins, makers, and worker users. Run Qryvanta yourself, define your data
          model, publish changes, and operate with clear checks.
        </p>
        <div className="flex flex-wrap items-center justify-center gap-3">
          <Link
            href="/docs"
            className="rounded-lg bg-fd-primary px-4 py-2 text-sm font-semibold text-fd-primary-foreground"
          >
            Read Docs
          </Link>
          <Link
            href="/docs/quickstart"
            className="rounded-lg border border-fd-border bg-white/80 px-4 py-2 text-sm font-semibold text-fd-foreground"
          >
            Open Quickstart
          </Link>
        </div>
      </div>

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        <Link href="/docs/quickstart" className="docs-home-card group rounded-xl p-4 transition-colors">
          <p className="text-xs font-semibold uppercase tracking-[0.14em] text-fd-muted-foreground">First Run</p>
          <h2 className="mt-2 text-lg font-semibold text-fd-foreground">Quickstart</h2>
          <p className="mt-2 text-sm text-fd-muted-foreground">Install, run infrastructure, and open your first product surface.</p>
        </Link>

        <Link href="/docs/workspace" className="docs-home-card group rounded-xl p-4 transition-colors">
          <p className="text-xs font-semibold uppercase tracking-[0.14em] text-fd-muted-foreground">Role Guides</p>
          <h2 className="mt-2 text-lg font-semibold text-fd-foreground">Workspace</h2>
          <p className="mt-2 text-sm text-fd-muted-foreground">Admin, Maker, and Worker workflows with clear task boundaries.</p>
        </Link>

        <Link href="/docs/concepts" className="docs-home-card group rounded-xl p-4 transition-colors">
          <p className="text-xs font-semibold uppercase tracking-[0.14em] text-fd-muted-foreground">Product Model</p>
          <h2 className="mt-2 text-lg font-semibold text-fd-foreground">Concepts</h2>
          <p className="mt-2 text-sm text-fd-muted-foreground">Metadata, publishing, runtime records, and workflow basics.</p>
        </Link>

        <Link href="/docs/operations" className="docs-home-card group rounded-xl p-4 transition-colors">
          <p className="text-xs font-semibold uppercase tracking-[0.14em] text-fd-muted-foreground">Self-Hosting</p>
          <h2 className="mt-2 text-lg font-semibold text-fd-foreground">Operations</h2>
          <p className="mt-2 text-sm text-fd-muted-foreground">Configuration, email delivery, monitoring, and incident response.</p>
        </Link>
      </div>

      <div className="docs-home-card rounded-2xl p-5 md:p-6">
        <h3 className="text-lg font-semibold text-fd-foreground">Product Boundary</h3>
        <p className="mt-2 text-sm text-fd-muted-foreground">
          `qryvanta.org` docs cover the open-source product and self-hosting path. Managed cloud platform behavior is documented on `qryvanta.com`.
        </p>
      </div>
    </div>
  );
}

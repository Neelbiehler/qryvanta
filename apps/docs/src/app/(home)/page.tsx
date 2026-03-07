import Link from 'next/link';
import { ArrowRight, BookOpenText, Boxes, HardDriveDownload, ShieldCheck } from 'lucide-react';

const tracks = [
  {
    href: '/docs',
    eyebrow: 'Start here',
    title: 'Learn the product in order',
    body: 'Start with the overview, then move through quickstart, workspace, concepts, and operations.',
    icon: BookOpenText,
  },
  {
    href: '/docs/quickstart/first-app',
    eyebrow: 'Hands-on guide',
    title: 'Build one app end to end',
    body: 'Create an entity, publish it, open Worker Apps, and verify the runtime result.',
    icon: Boxes,
  },
  {
    href: '/docs/operations/self-hosting',
    eyebrow: 'Self-hosting',
    title: 'Run the stack yourself',
    body: 'Use the operations docs for deployment shape, secret handling, backups, and worker runtime.',
    icon: HardDriveDownload,
  },
  {
    href: '/docs/operations/security-hardening',
    eyebrow: 'Security',
    title: 'Review the security model',
    body: 'Read the docs on RBAC, tenant isolation, security events, ingress checks, and secret rotation.',
    icon: ShieldCheck,
  },
];

const proofPoints = [
  {
    title: 'One contract source',
    body: 'Rust request and response DTOs generate the TypeScript transport types instead of drifting into parallel definitions.',
  },
  {
    title: 'Published metadata rules the runtime',
    body: 'Worker behavior changes when metadata is published, not when drafts are edited.',
  },
  {
    title: 'Operator work is documented',
    body: 'Security, migration, observability, and workflow operations are part of the product manual instead of tribal knowledge.',
  },
];

const audienceTracks = [
  {
    href: '/docs/workspace/admin-center',
    title: 'Admins',
    body: 'Role design, invite policy, audit visibility, and tenant controls.',
  },
  {
    href: '/docs/workspace/maker-center',
    title: 'Makers',
    body: 'Entity modeling, forms, views, navigation, workflows, and publish checks.',
  },
  {
    href: '/docs/workspace/worker-apps',
    title: 'Runtime users',
    body: 'What published apps, forms, and permissions look like in daily use.',
  },
  {
    href: '/docs/operations/self-hosting',
    title: 'Operators',
    body: 'Deployment shape, security hardening, backups, worker runtime, and incident readiness.',
  },
];

export default function HomePage() {
  return (
    <div className="mx-auto flex w-full max-w-6xl flex-1 flex-col gap-8 px-6 pb-20 pt-12 md:gap-12 md:px-10 md:pt-16">
      <section className="docs-home-hero rounded-[1.75rem] px-6 py-8 md:px-10 md:py-10">
        <div className="grid gap-10 lg:grid-cols-[minmax(0,1.35fr)_minmax(18rem,0.9fr)] lg:items-end">
          <div className="relative z-10 max-w-3xl">
            <p className="docs-chip inline-flex rounded-full px-3 py-1 text-xs font-semibold uppercase tracking-[0.18em]">
              Open source docs for qryvanta.org
            </p>
            <h1 className="mt-4 max-w-2xl text-balance text-4xl font-semibold tracking-tight text-fd-foreground md:text-5xl">
              Documentation for running and changing Qryvanta
            </h1>
            <p className="mt-4 max-w-2xl text-pretty text-lg leading-8 text-fd-muted-foreground">
              Start here if you need to run the stack, model metadata, check what publish changes, or review the self-hosting path.
            </p>
            <div className="mt-6 flex flex-wrap items-center gap-3">
              <Link
                href="/docs"
                className="rounded-xl bg-fd-primary px-4 py-2.5 text-sm font-semibold text-fd-primary-foreground shadow-sm"
              >
                Open Docs
              </Link>
              <Link
                href="/docs/quickstart/first-app"
                className="rounded-xl border border-fd-border bg-white/85 px-4 py-2.5 text-sm font-semibold text-fd-foreground"
              >
                Follow the First App guide
              </Link>
            </div>
            <div className="mt-6 flex flex-wrap gap-2">
              <span className="docs-mini-pill">Rust-first contracts</span>
              <span className="docs-mini-pill">Metadata-driven runtime</span>
              <span className="docs-mini-pill">Self-hosting runbooks</span>
            </div>
          </div>

          <div className="docs-home-ledger relative z-10 rounded-[1.45rem] p-5">
            <p className="text-xs font-semibold uppercase tracking-[0.14em] text-fd-muted-foreground">Why this project reads well</p>
            <div className="mt-4 space-y-4">
              {proofPoints.map((point) => (
                <div key={point.title} className="docs-home-ledger-item">
                  <h2 className="text-base font-semibold tracking-tight text-fd-foreground">{point.title}</h2>
                  <p className="mt-1 text-sm leading-6 text-fd-muted-foreground">{point.body}</p>
                </div>
              ))}
            </div>
            <Link href="/docs/concepts/platform-architecture" className="docs-home-inline-link mt-5 inline-flex items-center gap-2">
              Open the architecture guide
              <ArrowRight className="size-4" />
            </Link>
          </div>
        </div>
      </section>

      <section className="grid gap-4 lg:grid-cols-[minmax(0,1.15fr)_minmax(18rem,0.85fr)]">
        <div className="docs-home-card rounded-[1.4rem] p-5 md:p-6">
          <p className="text-xs font-semibold uppercase tracking-[0.14em] text-fd-muted-foreground">Read in this order</p>
          <ol className="mt-4 grid gap-3 md:grid-cols-2">
            <li className="docs-home-step">
              <strong>1. Start Here</strong>
              <div>Learn what Qryvanta does today and which section fits your task.</div>
            </li>
            <li className="docs-home-step">
              <strong>2. Quickstart</strong>
              <div>Bring up the stack, sign in, and publish your first metadata change.</div>
            </li>
            <li className="docs-home-step">
              <strong>3. Workspace and Concepts</strong>
              <div>Learn how Admin, Maker, and Worker fit together and why publish boundaries matter.</div>
            </li>
            <li className="docs-home-step">
              <strong>4. Operations</strong>
              <div>Use the self-hosting runbooks for rollout, security checks, and observability.</div>
            </li>
          </ol>
        </div>

        <div className="docs-home-card rounded-[1.4rem] p-5 md:p-6">
          <p className="text-xs font-semibold uppercase tracking-[0.14em] text-fd-muted-foreground">Project boundary</p>
          <h2 className="mt-3 text-2xl font-semibold tracking-tight text-fd-foreground">This docs app explains the actual OSS product</h2>
          <p className="mt-3 text-sm leading-6 text-fd-muted-foreground">
            It covers product behavior, metadata rules, workflow runtime, and self-hosting. Maintainer-only process docs stay in the repository `docs/` directory.
          </p>
        </div>
      </section>

      <section className="grid gap-4 md:grid-cols-2">
        {tracks.map((track) => {
          const Icon = track.icon;

          return (
            <Link key={track.href} href={track.href} className="docs-home-card group rounded-2xl p-5 transition-colors">
              <div className="flex items-start justify-between gap-4">
                <div>
                  <p className="text-xs font-semibold uppercase tracking-[0.14em] text-fd-muted-foreground">{track.eyebrow}</p>
                  <h2 className="mt-2 text-xl font-semibold tracking-tight text-fd-foreground">{track.title}</h2>
                </div>
                <span className="rounded-xl border border-fd-border bg-white/80 p-2">
                  <Icon className="size-5 text-fd-primary" />
                </span>
              </div>
              <p className="mt-3 max-w-xl text-sm leading-6 text-fd-muted-foreground">{track.body}</p>
              <div className="mt-4 inline-flex items-center gap-2 text-sm font-semibold text-fd-foreground">
                Open guide
                <ArrowRight className="size-4 transition group-hover:translate-x-0.5 group-hover:text-fd-primary" />
              </div>
            </Link>
          );
        })}
      </section>

      <section className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        {audienceTracks.map((track) => (
          <Link key={track.href} href={track.href} className="docs-home-card rounded-[1.35rem] p-5">
            <p className="text-sm font-semibold tracking-tight text-fd-foreground">{track.title}</p>
            <p className="mt-2 text-sm leading-6 text-fd-muted-foreground">{track.body}</p>
          </Link>
        ))}
      </section>

      <section className="docs-home-card rounded-[1.45rem] p-5 md:p-6">
        <div className="grid gap-4 lg:grid-cols-[minmax(0,1.3fr)_minmax(16rem,0.7fr)] lg:items-center">
          <div>
            <p className="text-xs font-semibold uppercase tracking-[0.14em] text-fd-muted-foreground">Current status</p>
            <h2 className="mt-3 text-2xl font-semibold tracking-tight text-fd-foreground">
              Built for evaluation, local development, and serious self-hosting preparation
            </h2>
            <p className="mt-3 max-w-3xl text-sm leading-6 text-fd-muted-foreground">
              Qryvanta is still in active development. The docs are written to help people understand the project deeply before trusting it, changing it, or operating it.
            </p>
          </div>
          <div className="docs-home-sequence rounded-[1.2rem] p-4">
            <p className="text-xs font-semibold uppercase tracking-[0.14em] text-fd-muted-foreground">Good starting points</p>
            <div className="mt-3 grid gap-2">
              <Link className="docs-sidebar-quicklink" href="/docs/quickstart">
                Quickstart
              </Link>
              <Link className="docs-sidebar-quicklink" href="/docs/concepts/platform-architecture">
                Platform Architecture
              </Link>
              <Link className="docs-sidebar-quicklink" href="/docs/operations/security-hardening">
                Security Hardening
              </Link>
            </div>
          </div>
        </div>
      </section>
    </div>
  );
}

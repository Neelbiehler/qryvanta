import Link from 'next/link';
import type { ComponentPropsWithoutRef, ReactNode } from 'react';
import { ArrowRight, CheckCircle2, Info, TriangleAlert } from 'lucide-react';
import { cn } from '@/lib/cn';

type Tone = 'note' | 'tip' | 'warn';

const toneClassName: Record<Tone, string> = {
  note: 'border-fd-border/85 bg-white/92 text-fd-foreground',
  tip: 'border-emerald-200/90 bg-[linear-gradient(180deg,rgba(236,253,245,0.96),rgba(220,252,231,0.94))] text-emerald-950',
  warn: 'border-amber-300/90 bg-[linear-gradient(180deg,rgba(255,251,235,0.98),rgba(254,243,199,0.95))] text-amber-950',
};

const toneIcon: Record<Tone, ReactNode> = {
  note: <Info className="mt-0.5 size-4 shrink-0" />,
  tip: <CheckCircle2 className="mt-0.5 size-4 shrink-0" />,
  warn: <TriangleAlert className="mt-0.5 size-4 shrink-0" />,
};

export function DocCallout({
  title,
  tone = 'note',
  children,
}: {
  title: string;
  tone?: Tone;
  children: ReactNode;
}) {
  return (
    <div
      className={cn(
        'my-6 rounded-[1.35rem] border px-4 py-4 shadow-[0_14px_30px_-24px_rgba(26,55,40,0.35)]',
        toneClassName[tone],
      )}
    >
      <div className="flex gap-3">
        {toneIcon[tone]}
        <div className="min-w-0">
          <p className="text-sm font-semibold tracking-tight">{title}</p>
          <div className="mt-1 text-sm leading-6 opacity-90">{children}</div>
        </div>
      </div>
    </div>
  );
}

export function DocCardGrid({
  children,
  columns = 2,
}: {
  children: ReactNode;
  columns?: 2 | 3;
}) {
  return (
    <div
      className={cn(
        'my-6 grid gap-4',
        columns === 3 ? 'md:grid-cols-2 xl:grid-cols-3' : 'md:grid-cols-2',
      )}
    >
      {children}
    </div>
  );
}

type DocCardProps = {
  title: string;
  eyebrow?: string;
  href?: string;
  children: ReactNode;
};

export function DocCard({ title, eyebrow, href, children }: DocCardProps) {
  const className =
    'group block rounded-[1.35rem] border border-fd-border/80 bg-[linear-gradient(180deg,rgba(255,255,255,0.97),rgba(245,252,248,0.95))] p-5 shadow-[0_18px_38px_-28px_rgba(27,67,45,0.45)] transition duration-200 hover:-translate-y-0.5 hover:border-fd-primary/40 hover:bg-white';

  const content = (
    <>
      {eyebrow ? (
        <p className="text-[0.72rem] font-semibold uppercase tracking-[0.16em] text-fd-muted-foreground">
          {eyebrow}
        </p>
      ) : null}
      <div className="mt-2 flex items-start justify-between gap-3">
        <h3 className="text-lg font-semibold tracking-tight text-fd-foreground">{title}</h3>
        {href ? (
          <ArrowRight className="mt-0.5 size-4 shrink-0 text-fd-muted-foreground transition group-hover:translate-x-0.5 group-hover:text-fd-primary" />
        ) : null}
      </div>
      <div className="mt-2 text-sm leading-6 text-fd-muted-foreground">{children}</div>
    </>
  );

  if (!href) {
    return <section className={className}>{content}</section>;
  }

  const isExternal = href.startsWith('http://') || href.startsWith('https://');
  if (isExternal) {
    return (
      <a className={className} href={href} rel="noreferrer noopener" target="_blank">
        {content}
      </a>
    );
  }

  return (
    <Link className={className} href={href}>
      {content}
    </Link>
  );
}

export function AudienceTags({ children }: { children: ReactNode }) {
  return <div className="my-5 flex flex-wrap gap-2">{children}</div>;
}

export function AudienceTag({ children }: { children: ReactNode }) {
  return (
    <span className="rounded-full border border-fd-border bg-[linear-gradient(180deg,rgba(255,255,255,0.92),rgba(229,242,234,0.88))] px-3 py-1 text-xs font-medium text-fd-foreground shadow-[0_8px_16px_-14px_rgba(21,32,38,0.45)]">
      {children}
    </span>
  );
}

export function DocSummary({ children }: { children: ReactNode }) {
  return <div className="my-6 grid gap-3 md:grid-cols-3">{children}</div>;
}

export function DocSummaryItem({
  label,
  children,
}: {
  label: string;
  children: ReactNode;
}) {
  return (
    <section className="rounded-[1.25rem] border border-fd-border/80 bg-[linear-gradient(180deg,rgba(255,255,255,0.96),rgba(243,250,246,0.93))] px-4 py-4 shadow-[0_16px_34px_-28px_rgba(27,67,45,0.28)]">
      <p className="text-[0.72rem] font-semibold uppercase tracking-[0.16em] text-fd-muted-foreground">
        {label}
      </p>
      <div className="mt-2 text-sm leading-6 text-fd-foreground">{children}</div>
    </section>
  );
}

export function Checklist(props: ComponentPropsWithoutRef<'ul'>) {
  return (
    <ul
      {...props}
      className={cn(
        'my-5 space-y-3 rounded-[1.25rem] border border-fd-border/80 bg-[linear-gradient(180deg,rgba(255,255,255,0.95),rgba(243,250,246,0.92))] px-4 py-4 text-sm leading-6 text-fd-foreground shadow-[0_14px_28px_-26px_rgba(27,67,45,0.24)]',
        '[&_li]:relative [&_li]:pl-7 [&_li]:before:absolute [&_li]:before:left-0 [&_li]:before:top-1 [&_li]:before:size-4 [&_li]:before:rounded-full [&_li]:before:border [&_li]:before:border-fd-primary/35 [&_li]:before:bg-fd-primary/10',
        '[&_li]:after:absolute [&_li]:after:left-[0.3rem] [&_li]:after:top-[0.55rem] [&_li]:after:h-1.5 [&_li]:after:w-2.5 [&_li]:after:rotate-[-45deg] [&_li]:after:border-b-2 [&_li]:after:border-l-2 [&_li]:after:border-fd-primary',
        props.className,
      )}
    />
  );
}

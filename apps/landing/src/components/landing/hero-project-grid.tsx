import Link from "next/link";
import { Cloud } from "lucide-react";

import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  StatusBadge,
  buttonVariants,
} from "@qryvanta/ui";

type HeroProjectGridProps = {
  docsUrl: string;
  githubUrl: string;
};

export function HeroProjectGrid({ docsUrl, githubUrl }: HeroProjectGridProps) {
  return (
    <section className="animate-rise-delay">
      <div className="grid gap-4 lg:grid-cols-[1.2fr_0.8fr]">
        <div className="rounded-3xl border border-emerald-100/85 bg-white/88 p-7 shadow-sm backdrop-blur-sm md:p-8">
          <StatusBadge tone="success">Open Source Project Preview</StatusBadge>
          <h1 className="landing-display mt-4 text-balance text-4xl leading-tight text-slate-900 md:text-5xl md:leading-[1.05]">
            Build metadata-driven business systems in the open.
          </h1>
          <p className="mt-5 max-w-xl text-pretty text-base text-slate-600 md:text-lg">
            Qryvanta.org is the project home for architecture notes and
            implementation progress. Browse a live preview of the platform
            surfaces built in the open.
          </p>

          <div className="mt-5 rounded-xl border border-sky-100 bg-sky-50/70 p-3 text-sm text-slate-700">
            <p className="flex items-start gap-2">
              <Cloud className="mt-0.5 h-4 w-4 shrink-0 text-sky-700" />
              <span>
                This site covers the project roadmap and documentation. The
                managed cloud offering lives on
                <span className="ml-1 font-semibold">qryvanta.com</span>.
              </span>
            </p>
          </div>

          <div className="mt-7 flex flex-wrap gap-3">
            <Link href={docsUrl} className={buttonVariants({ size: "lg" })}>
              Read Documentation
            </Link>
            <Link
              href={githubUrl}
              className={buttonVariants({ variant: "outline", size: "lg" })}
            >
              View on GitHub
            </Link>
          </div>
        </div>

        <Card className="border-emerald-100/90 bg-white/90">
          <CardHeader>
            <div className="flex items-center justify-between gap-2">
              <CardTitle className="landing-display text-2xl">Project Hub</CardTitle>
              <StatusBadge tone="neutral">Start Here</StatusBadge>
            </div>
          </CardHeader>
          <CardContent className="space-y-3">
            <p className="text-sm text-slate-600">
              Read the architecture and follow delivery phases. Then pick up an
              open issue on GitHub.
            </p>

            <article className="rounded-lg border border-emerald-100/90 bg-emerald-50/70 p-3">
              <p className="text-xs font-semibold uppercase tracking-[0.14em] text-slate-500">
                Read first
              </p>
              <p className="mt-1 text-sm text-slate-700">
                Quickstart, architecture, platform boundaries, and operational
                guides.
              </p>
              <Link
                href={docsUrl}
                className={`${buttonVariants({ variant: "outline", size: "sm" })} mt-2`}
              >
                Open docs
              </Link>
            </article>

            <article className="rounded-lg border border-emerald-100/90 bg-emerald-50/70 p-3">
              <p className="text-xs font-semibold uppercase tracking-[0.14em] text-slate-500">
                Build with us
              </p>
              <p className="mt-1 text-sm text-slate-700">
                Browse issues, propose changes, and submit pull requests to move
                the project forward.
              </p>
              <Link
                href={`${githubUrl}/issues`}
                className={`${buttonVariants({ variant: "outline", size: "sm" })} mt-2`}
              >
                Open issues
              </Link>
            </article>

            <article className="rounded-lg border border-sky-100/90 bg-sky-50/70 p-3">
              <p className="text-xs font-semibold uppercase tracking-[0.14em] text-slate-500">
                Project structure
              </p>
              <p className="mt-1 text-sm text-slate-700">
                Read the layered architecture and surface tracks before starting
                on implementation.
              </p>
              <Link
                href="#architecture"
                className={`${buttonVariants({ variant: "outline", size: "sm" })} mt-2`}
              >
                View architecture
              </Link>
            </article>
          </CardContent>
        </Card>
      </div>
    </section>
  );
}

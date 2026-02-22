import Link from "next/link";
import { ArrowUpRight } from "lucide-react";

import { Card, CardContent, buttonVariants } from "@qryvanta/ui";

type ContributeSectionProps = {
  docsUrl: string;
  githubUrl: string;
};

export function ContributeSection({ docsUrl, githubUrl }: ContributeSectionProps) {
  return (
    <section className="animate-rise-delay mt-12">
      <Card className="border-emerald-200 bg-white/92">
        <CardContent className="flex flex-col items-start justify-between gap-4 p-6 md:flex-row md:items-center">
          <div>
            <p className="text-xs font-semibold uppercase tracking-[0.18em] text-emerald-700">
              Ready to contribute?
            </p>
            <p className="mt-2 text-sm text-slate-600">
              Follow the docs, review architecture, and contribute directly to
              the repository.
            </p>
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <Link href={docsUrl} className={buttonVariants({ size: "lg" })}>
              Read Docs
            </Link>
            <Link
              href={githubUrl}
              className={buttonVariants({ variant: "outline", size: "lg" })}
            >
              Contribute on GitHub
              <ArrowUpRight className="h-4 w-4" />
            </Link>
          </div>
        </CardContent>
      </Card>
    </section>
  );
}

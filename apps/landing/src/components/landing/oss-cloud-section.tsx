import Link from "next/link";
import { ArrowUpRight, Cloud, HardDriveDownload } from "lucide-react";

import { Card, CardContent, buttonVariants } from "@qryvanta/ui";

type OssCloudSectionProps = {
  docsUrl: string;
  cloudUrl: string;
};

export function OssCloudSection({ docsUrl, cloudUrl }: OssCloudSectionProps) {
  return (
    <section className="animate-rise-delay mt-12">
      <Card className="border-emerald-200 bg-white/92">
        <CardContent className="grid gap-4 p-6 md:grid-cols-[1.15fr_0.85fr]">
          <div>
            <p className="text-xs font-semibold uppercase tracking-[0.18em] text-emerald-700">
              Open source first
            </p>
            <h3 className="landing-display mt-2 text-2xl text-slate-900">
              Deploy yourself, or choose managed cloud when it helps.
            </h3>
            <p className="mt-3 text-sm text-slate-600">
              Qryvanta.org stays focused on the OSS project. For teams that want
              managed infrastructure, updates, and support, Qryvanta Cloud is
              available without changing the platform story.
            </p>
          </div>

          <div className="grid gap-3">
            <article className="rounded-xl border border-emerald-100 bg-emerald-50/70 p-3">
              <div className="flex items-center gap-2">
                <HardDriveDownload className="h-4 w-4 text-emerald-700" />
                <p className="text-sm font-semibold text-slate-900">Self-host path</p>
              </div>
              <p className="mt-1 text-xs text-slate-600">
                Use docs and architecture guides to run Qryvanta on your own
                stack.
              </p>
              <Link
                href={docsUrl}
                className={`${buttonVariants({ variant: "outline", size: "sm" })} mt-2`}
              >
                Self-host docs
              </Link>
            </article>

            <article className="rounded-xl border border-sky-100 bg-sky-50/70 p-3">
              <div className="flex items-center gap-2">
                <Cloud className="h-4 w-4 text-sky-700" />
                <p className="text-sm font-semibold text-slate-900">
                  Managed cloud option
                </p>
              </div>
              <p className="mt-1 text-xs text-slate-600">
                Explore the hosted option for faster rollout and managed
                operations.
              </p>
              <Link href={cloudUrl} className={`${buttonVariants({ size: "sm" })} mt-2`}>
                Visit qryvanta.com
                <ArrowUpRight className="h-4 w-4" />
              </Link>
            </article>
          </div>
        </CardContent>
      </Card>
    </section>
  );
}

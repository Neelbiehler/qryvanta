import Link from "next/link";

import { StatusBadge, buttonVariants } from "@qryvanta/ui";

import { NavOrb } from "@/components/nav-orb";

type HeaderNavProps = {
  githubUrl: string;
};

export function HeaderNav({ githubUrl }: HeaderNavProps) {
  return (
    <header className="animate-rise">
      <nav className="mb-10 flex items-center justify-between gap-3 rounded-2xl border border-emerald-100/80 bg-white/80 px-4 py-3 backdrop-blur-sm md:px-5">
        <div className="flex items-center gap-2">
          <NavOrb className="shrink-0" />
          <span className="text-sm font-semibold tracking-[0.18em] text-slate-700">
            QRYVANTA
          </span>
          <StatusBadge tone="success" className="hidden sm:inline-flex">
            qryvanta.org OSS
          </StatusBadge>
        </div>
        <div className="flex items-center gap-2">
          <Link
            href="#architecture"
            className={buttonVariants({ variant: "ghost", size: "sm" })}
          >
            Architecture
          </Link>
          <Link href={githubUrl} className={buttonVariants({ size: "sm" })}>
            GitHub
          </Link>
        </div>
      </nav>
    </header>
  );
}

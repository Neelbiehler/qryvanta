import type { ReactNode } from "react";

type WorkerCommandRibbonProps = {
  eyebrow?: string;
  title: string;
  subtitle: string;
  badges?: ReactNode;
  actions?: ReactNode;
};

export function WorkerCommandRibbon({
  eyebrow = "Command Bar",
  title,
  subtitle,
  badges,
  actions,
}: WorkerCommandRibbonProps) {
  return (
    <div className="sticky top-0 z-10 border-b border-emerald-100 bg-white px-4 py-2.5 shadow-sm">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div className="pl-10 xl:pl-0">
          <p className="text-[10px] font-semibold uppercase tracking-[0.14em] text-emerald-700">
            {eyebrow}
          </p>
          <p className="text-sm font-semibold text-zinc-900">{title}</p>
          <p className="font-mono text-[10px] text-zinc-400">{subtitle}</p>
        </div>
        <div className="flex flex-wrap items-center gap-2">
          {badges}
          {actions}
        </div>
      </div>
    </div>
  );
}

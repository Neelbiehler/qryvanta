import * as React from "react";

import { cn } from "../lib/cn";

export type EmptyStateProps = React.HTMLAttributes<HTMLDivElement> & {
  icon?: React.ReactNode;
  title: React.ReactNode;
  description?: React.ReactNode;
  action?: React.ReactNode;
};

export function EmptyState({
  className,
  icon,
  title,
  description,
  action,
  ...props
}: EmptyStateProps) {
  return (
    <div
      className={cn(
        "flex min-h-52 w-full flex-col items-center justify-center rounded-lg border border-dashed border-emerald-200 bg-emerald-50/40 px-6 py-8 text-center",
        className,
      )}
      {...props}
    >
      {icon ? (
        <div className="mb-3 inline-flex size-10 items-center justify-center rounded-full bg-white text-emerald-700 shadow-sm">
          {icon}
        </div>
      ) : null}
      <h3 className="text-base font-semibold text-zinc-900">{title}</h3>
      {description ? <p className="mt-2 max-w-xl text-sm text-zinc-600">{description}</p> : null}
      {action ? <div className="mt-4">{action}</div> : null}
    </div>
  );
}

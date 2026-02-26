import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";

import { cn } from "../lib/cn";

const statusBadgeVariants = cva(
  "inline-flex items-center gap-1.5 rounded-full px-2.5 py-1 text-[11px] font-semibold uppercase tracking-[0.12em]",
  {
    variants: {
      tone: {
        neutral: "bg-zinc-100 text-zinc-700",
        success: "bg-emerald-100 text-emerald-800",
        warning: "bg-amber-100 text-amber-800",
        info: "bg-sky-100 text-sky-800",
        critical: "bg-red-100 text-red-800",
      },
    },
    defaultVariants: {
      tone: "neutral",
    },
  },
);

export type StatusBadgeProps = React.HTMLAttributes<HTMLSpanElement> &
  VariantProps<typeof statusBadgeVariants> & {
    icon?: React.ReactNode;
    dot?: boolean;
  };

export function StatusBadge({
  className,
  tone,
  icon,
  dot = false,
  children,
  ...props
}: StatusBadgeProps) {
  return (
    <span className={cn(statusBadgeVariants({ tone }), className)} {...props}>
      {dot ? <span className="inline-block size-1.5 rounded-full bg-current/70" aria-hidden /> : null}
      {icon ? <span className="inline-flex size-3.5 items-center justify-center">{icon}</span> : null}
      {children}
    </span>
  );
}

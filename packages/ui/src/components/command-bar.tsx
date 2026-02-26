import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";

import { cn } from "../lib/cn";

const commandBarActionVariants = cva(
  "inline-flex h-9 items-center gap-2 rounded-md border px-3 text-sm font-medium transition-colors disabled:pointer-events-none disabled:opacity-50",
  {
    variants: {
      variant: {
        default:
          "border-emerald-200 bg-white text-zinc-700 hover:border-emerald-300 hover:bg-emerald-50",
        primary:
          "border-emerald-700 bg-emerald-700 text-white hover:border-emerald-800 hover:bg-emerald-800",
        danger: "border-red-200 bg-red-50 text-red-700 hover:border-red-300 hover:bg-red-100",
      },
    },
    defaultVariants: {
      variant: "default",
    },
  },
);

export function CommandBar({
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      className={cn(
        "flex min-h-12 items-center gap-2 border-b border-[var(--command-bar-border,#d8ebdf)] bg-[var(--command-bar-bg,#ffffff)] px-3 py-2",
        className,
      )}
      {...props}
    />
  );
}

export function CommandBarGroup({
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement>) {
  return <div className={cn("flex items-center gap-2", className)} {...props} />;
}

export type CommandBarActionProps = React.ButtonHTMLAttributes<HTMLButtonElement> &
  VariantProps<typeof commandBarActionVariants> & {
    icon?: React.ReactNode;
  };

export function CommandBarAction({
  className,
  variant,
  icon,
  children,
  ...props
}: CommandBarActionProps) {
  return (
    <button
      type="button"
      className={cn(commandBarActionVariants({ variant }), className)}
      {...props}
    >
      {icon ? <span className="inline-flex size-4 items-center justify-center">{icon}</span> : null}
      <span>{children}</span>
    </button>
  );
}

export function CommandBarSeparator({
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      role="separator"
      aria-orientation="vertical"
      className={cn("mx-1 h-6 w-px bg-emerald-200", className)}
      {...props}
    />
  );
}

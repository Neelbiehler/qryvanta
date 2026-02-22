import * as React from "react";

import { cva, type VariantProps } from "class-variance-authority";

import { cn } from "../lib/cn";

const noticeVariants = cva("rounded-md border px-3 py-2 text-sm", {
  variants: {
    tone: {
      neutral: "border-zinc-200 bg-zinc-50 text-zinc-700",
      success: "border-emerald-200 bg-emerald-50 text-emerald-700",
      warning: "border-amber-200 bg-amber-50 text-amber-800",
      error: "border-red-200 bg-red-50 text-red-700",
    },
  },
  defaultVariants: {
    tone: "neutral",
  },
});

export type NoticeProps = React.HTMLAttributes<HTMLParagraphElement> &
  VariantProps<typeof noticeVariants>;

export function Notice({ tone, className, ...props }: NoticeProps) {
  return <p className={cn(noticeVariants({ tone }), className)} {...props} />;
}

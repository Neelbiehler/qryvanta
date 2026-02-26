"use client";

import * as React from "react";
import { cva } from "class-variance-authority";

import { cn } from "../lib/cn";
import { useToast, type ToastTone } from "./use-toast";

const toastVariants = cva("rounded-md border p-3 shadow-lg", {
  variants: {
    tone: {
      neutral: "border-zinc-200 bg-white text-zinc-800",
      success: "border-emerald-200 bg-emerald-50 text-emerald-900",
      warning: "border-amber-200 bg-amber-50 text-amber-900",
      error: "border-red-200 bg-red-50 text-red-900",
    } satisfies Record<ToastTone, string>,
  },
  defaultVariants: {
    tone: "neutral",
  },
});

export type ToasterProps = React.HTMLAttributes<HTMLDivElement> & {
  position?: "top-right" | "top-left" | "bottom-right" | "bottom-left";
};

export function Toaster({
  className,
  position = "top-right",
  ...props
}: ToasterProps) {
  const { toasts, dismissToast } = useToast();

  const positionClassName =
    position === "top-right"
      ? "right-4 top-4"
      : position === "top-left"
        ? "left-4 top-4"
        : position === "bottom-right"
          ? "bottom-4 right-4"
          : "bottom-4 left-4";

  return (
    <div
      className={cn(
        "pointer-events-none fixed z-[100] flex w-full max-w-sm flex-col gap-2",
        positionClassName,
        className,
      )}
      aria-live="polite"
      aria-atomic="false"
      {...props}
    >
      {toasts.map((toastItem) => (
        <div
          key={toastItem.id}
          role={toastItem.tone === "error" ? "alert" : "status"}
          className={cn("pointer-events-auto", toastVariants({ tone: toastItem.tone }))}
        >
          <div className="flex items-start justify-between gap-3">
            <div className="space-y-1">
              {toastItem.title ? <p className="text-sm font-semibold">{toastItem.title}</p> : null}
              {toastItem.description ? (
                <p className="text-sm text-current/85">{toastItem.description}</p>
              ) : null}
            </div>
            <button
              type="button"
              className="inline-flex size-6 items-center justify-center rounded text-current/70 transition-colors hover:bg-black/5 hover:text-current"
              onClick={() => dismissToast(toastItem.id)}
              aria-label="Dismiss notification"
            >
              ×
            </button>
          </div>
        </div>
      ))}
    </div>
  );
}

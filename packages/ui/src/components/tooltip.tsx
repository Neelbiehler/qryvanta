import * as React from "react";

import { cn } from "../lib/cn";

type TooltipContextValue = {
  contentId: string;
};

const TooltipContext = React.createContext<TooltipContextValue | null>(null);

function useTooltipContext() {
  const context = React.useContext(TooltipContext);
  if (!context) {
    throw new Error("Tooltip components must be wrapped in Tooltip");
  }

  return context;
}

export function Tooltip({
  className,
  ...props
}: React.HTMLAttributes<HTMLSpanElement>) {
  const contentId = React.useId();

  return (
    <TooltipContext.Provider value={{ contentId }}>
      <span className={cn("group/tooltip relative inline-flex", className)} {...props} />
    </TooltipContext.Provider>
  );
}

export function TooltipTrigger({
  children,
}: {
  children: React.ReactNode;
}) {
  const { contentId } = useTooltipContext();

  if (!React.isValidElement<React.HTMLAttributes<HTMLElement>>(children)) {
    return <>{children}</>;
  }

  const childProps = children.props;
  return React.cloneElement(children, {
    "aria-describedby": childProps["aria-describedby"]
      ? `${childProps["aria-describedby"]} ${contentId}`
      : contentId,
    className: cn("peer/tooltip", childProps.className),
  });
}

export type TooltipContentProps = React.HTMLAttributes<HTMLSpanElement> & {
  side?: "top" | "right" | "bottom" | "left";
};

export function TooltipContent({
  className,
  side = "top",
  ...props
}: TooltipContentProps) {
  const { contentId } = useTooltipContext();

  const sideClassName =
    side === "top"
      ? "bottom-full left-1/2 mb-2 -translate-x-1/2"
      : side === "right"
        ? "left-full top-1/2 ml-2 -translate-y-1/2"
        : side === "bottom"
          ? "left-1/2 top-full mt-2 -translate-x-1/2"
          : "right-full top-1/2 mr-2 -translate-y-1/2";

  return (
    <span
      id={contentId}
      role="tooltip"
      className={cn(
        "pointer-events-none absolute z-50 rounded-md bg-zinc-900 px-2 py-1 text-xs text-white opacity-0 shadow-md transition-opacity duration-150 group-hover/tooltip:opacity-100 group-focus-within/tooltip:opacity-100",
        sideClassName,
        className,
      )}
      {...props}
    />
  );
}

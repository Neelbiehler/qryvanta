import * as React from "react";

import { cn } from "../lib/cn";

type PropertyPanelContextValue = {
  title?: React.ReactNode;
  onOpenChange?: (open: boolean) => void;
};

const PropertyPanelContext = React.createContext<PropertyPanelContextValue | null>(null);

function usePropertyPanelContext() {
  const context = React.useContext(PropertyPanelContext);
  if (!context) {
    throw new Error("Property panel components must be wrapped in PropertyPanel");
  }

  return context;
}

export type PropertyPanelProps = React.HTMLAttributes<HTMLElement> & {
  open: boolean;
  onOpenChange?: (open: boolean) => void;
  title?: React.ReactNode;
  width?: number | string;
};

function resolvePanelWidth(width: number | string) {
  if (typeof width === "number") {
    return `${width}px`;
  }

  return width;
}

export function PropertyPanel({
  className,
  children,
  open,
  onOpenChange,
  title,
  width = 360,
  ...props
}: PropertyPanelProps) {
  return (
    <PropertyPanelContext.Provider value={{ onOpenChange, title }}>
      <aside
        data-state={open ? "open" : "closed"}
        className={cn(
          "relative h-full shrink-0 overflow-hidden border-l border-[var(--panel-border,#d8ebdf)] bg-[var(--panel-bg,#ffffff)] transition-[width,opacity] duration-200",
          open ? "opacity-100" : "pointer-events-none opacity-0",
          className,
        )}
        style={{ width: open ? resolvePanelWidth(width) : 0 }}
        {...props}
      >
        <div className="flex h-full min-w-0 flex-col">{children}</div>
      </aside>
    </PropertyPanelContext.Provider>
  );
}

export function PropertyPanelHeader({
  className,
  children,
  ...props
}: React.HTMLAttributes<HTMLDivElement>) {
  const { title, onOpenChange } = usePropertyPanelContext();

  return (
    <div
      className={cn(
        "flex min-h-12 items-center justify-between gap-3 border-b border-[var(--panel-border,#d8ebdf)] bg-[var(--panel-header-bg,#f5faf7)] px-4 py-3",
        className,
      )}
      {...props}
    >
      <div className="min-w-0 truncate text-sm font-semibold text-zinc-800">{title}</div>
      <div className="ml-auto flex items-center gap-2">
        {children}
        {onOpenChange ? (
          <button
            type="button"
            className="inline-flex size-8 items-center justify-center rounded-md text-zinc-500 transition-colors hover:bg-zinc-100 hover:text-zinc-700"
            onClick={() => onOpenChange(false)}
            aria-label="Close panel"
          >
            ×
          </button>
        ) : null}
      </div>
    </div>
  );
}

export function PropertyPanelContent({
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement>) {
  return <div className={cn("min-h-0 flex-1 overflow-y-auto p-4", className)} {...props} />;
}

export type PropertyPanelSectionProps = React.HTMLAttributes<HTMLDivElement> & {
  title: React.ReactNode;
  collapsible?: boolean;
  defaultOpen?: boolean;
};

export function PropertyPanelSection({
  className,
  title,
  collapsible = false,
  defaultOpen = true,
  children,
  ...props
}: PropertyPanelSectionProps) {
  if (!collapsible) {
    return (
      <section className={cn("space-y-2 border-b border-emerald-100 pb-4", className)} {...props}>
        <h3 className="text-xs font-semibold uppercase tracking-[0.14em] text-zinc-500">{title}</h3>
        <div className="space-y-3">{children}</div>
      </section>
    );
  }

  return (
    <section className={cn("border-b border-emerald-100", className)} {...props}>
      <details className="group pb-2" open={defaultOpen}>
        <summary className="flex cursor-pointer list-none items-center justify-between gap-2 py-2 text-xs font-semibold uppercase tracking-[0.14em] text-zinc-500">
          <span>{title}</span>
          <span className="transition-transform group-open:rotate-180">▾</span>
        </summary>
        <div className="space-y-3 pb-2">{children}</div>
      </details>
    </section>
  );
}

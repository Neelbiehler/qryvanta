import * as React from "react";

import { cn } from "../lib/cn";

type TabsContextValue = {
  baseId: string;
  value: string;
  onValueChange: (value: string) => void;
};

const TabsContext = React.createContext<TabsContextValue | null>(null);

function useTabsContext() {
  const context = React.useContext(TabsContext);
  if (!context) {
    throw new Error("Tabs components must be wrapped in Tabs");
  }

  return context;
}

function toDomToken(input: string) {
  return input
    .toLowerCase()
    .trim()
    .replace(/[^a-z0-9_-]+/g, "-")
    .replace(/^-+|-+$/g, "");
}

export type TabsProps = React.HTMLAttributes<HTMLDivElement> & {
  value: string;
  onValueChange: (value: string) => void;
};

export function Tabs({
  className,
  value,
  onValueChange,
  ...props
}: TabsProps) {
  const baseId = React.useId();

  return (
    <TabsContext.Provider value={{ baseId, value, onValueChange }}>
      <div className={cn("w-full", className)} {...props} />
    </TabsContext.Provider>
  );
}

export function TabsList({
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      role="tablist"
      className={cn(
        "inline-flex h-10 items-center gap-1 rounded-md border border-emerald-200 bg-white p-1",
        className,
      )}
      {...props}
    />
  );
}

export type TabsTriggerProps = React.ButtonHTMLAttributes<HTMLButtonElement> & {
  value: string;
};

export function TabsTrigger({
  className,
  value,
  onClick,
  ...props
}: TabsTriggerProps) {
  const context = useTabsContext();
  const domToken = toDomToken(value);
  const selected = context.value === value;

  return (
    <button
      type="button"
      role="tab"
      id={`${context.baseId}-tab-${domToken}`}
      aria-controls={`${context.baseId}-panel-${domToken}`}
      aria-selected={selected}
      data-state={selected ? "active" : "inactive"}
      className={cn(
        "inline-flex min-w-16 items-center justify-center rounded-sm px-3 py-1.5 text-sm font-medium text-zinc-600 transition-colors hover:bg-emerald-50 hover:text-zinc-900 data-[state=active]:bg-emerald-700 data-[state=active]:text-white",
        className,
      )}
      onClick={(event) => {
        onClick?.(event);
        if (!event.defaultPrevented) {
          context.onValueChange(value);
        }
      }}
      {...props}
    />
  );
}

export type TabsContentProps = React.HTMLAttributes<HTMLDivElement> & {
  value: string;
};

export function TabsContent({
  className,
  value,
  ...props
}: TabsContentProps) {
  const context = useTabsContext();
  const domToken = toDomToken(value);
  const selected = context.value === value;

  return (
    <div
      role="tabpanel"
      id={`${context.baseId}-panel-${domToken}`}
      aria-labelledby={`${context.baseId}-tab-${domToken}`}
      hidden={!selected}
      className={cn("mt-4", className)}
      {...props}
    />
  );
}

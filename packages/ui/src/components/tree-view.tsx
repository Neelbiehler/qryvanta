"use client";

import * as React from "react";

import { cn } from "../lib/cn";

type TreeViewContextValue = {
  selectedValue: string | null;
  onSelectedValueChange?: (value: string) => void;
  expandedValues: string[];
  onExpandedValuesChange: (values: string[]) => void;
};

const TreeViewContext = React.createContext<TreeViewContextValue | null>(null);

function useTreeViewContext() {
  const context = React.useContext(TreeViewContext);
  if (!context) {
    throw new Error("Tree view components must be wrapped in TreeView");
  }

  return context;
}

type TreeViewProps = React.HTMLAttributes<HTMLUListElement> & {
  selectedValue?: string | null;
  onSelectedValueChange?: (value: string) => void;
  expandedValues?: string[];
  defaultExpandedValues?: string[];
  onExpandedValuesChange?: (values: string[]) => void;
};

export function TreeView({
  className,
  selectedValue = null,
  onSelectedValueChange,
  expandedValues,
  defaultExpandedValues,
  onExpandedValuesChange,
  children,
  onKeyDown,
  ...props
}: TreeViewProps) {
  const [internalExpandedValues, setInternalExpandedValues] = React.useState<string[]>(
    defaultExpandedValues ?? [],
  );

  const resolvedExpandedValues = expandedValues ?? internalExpandedValues;

  const setExpandedValues = React.useCallback(
    (values: string[]) => {
      if (expandedValues === undefined) {
        setInternalExpandedValues(values);
      }
      onExpandedValuesChange?.(values);
    },
    [expandedValues, onExpandedValuesChange],
  );

  return (
    <TreeViewContext.Provider
      value={{
        selectedValue,
        onSelectedValueChange,
        expandedValues: resolvedExpandedValues,
        onExpandedValuesChange: setExpandedValues,
      }}
    >
      <ul
        role="tree"
        className={cn("space-y-1", className)}
        onKeyDown={(event) => {
          onKeyDown?.(event);
          if (event.defaultPrevented) {
            return;
          }

          const rootElement = event.currentTarget;
          const visibleItems = Array.from(
            rootElement.querySelectorAll<HTMLElement>("[role='treeitem']"),
          ).filter((element) => element.offsetParent !== null);

          if (visibleItems.length === 0) {
            return;
          }

          const activeElement = document.activeElement as HTMLElement | null;
          const index = activeElement ? visibleItems.indexOf(activeElement) : -1;

          if (event.key === "ArrowDown") {
            event.preventDefault();
            const next = visibleItems[Math.min(index + 1, visibleItems.length - 1)] ?? visibleItems[0];
            next.focus();
            return;
          }

          if (event.key === "ArrowUp") {
            event.preventDefault();
            const next = visibleItems[Math.max(index - 1, 0)] ?? visibleItems[visibleItems.length - 1];
            next.focus();
            return;
          }

          if (event.key === "Enter") {
            event.preventDefault();
            activeElement?.click();
          }
        }}
        {...props}
      >
        {children}
      </ul>
    </TreeViewContext.Provider>
  );
}

export type TreeViewItemProps = React.HTMLAttributes<HTMLLIElement> & {
  value: string;
  label: React.ReactNode;
  icon?: React.ReactNode;
  expandable?: boolean;
  selected?: boolean;
};

export function TreeViewItem({
  className,
  value,
  label,
  icon,
  expandable,
  selected,
  children,
  ...props
}: TreeViewItemProps) {
  const context = useTreeViewContext();
  const isSelected = selected ?? context.selectedValue === value;
  const hasNestedItems = React.Children.count(children) > 0;
  const isExpandable = expandable ?? hasNestedItems;
  const isExpanded = context.expandedValues.includes(value);

  function setExpanded(nextExpanded: boolean) {
    if (!isExpandable) {
      return;
    }

    if (nextExpanded) {
      if (!context.expandedValues.includes(value)) {
        context.onExpandedValuesChange([...context.expandedValues, value]);
      }
      return;
    }

    context.onExpandedValuesChange(context.expandedValues.filter((entry) => entry !== value));
  }

  return (
    <li className={cn("min-w-0", className)} {...props}>
      <div className="flex min-w-0 items-center gap-1">
        {isExpandable ? (
          <button
            type="button"
            className="inline-flex size-6 items-center justify-center rounded text-zinc-500 hover:bg-zinc-100"
            aria-label={isExpanded ? "Collapse" : "Expand"}
            onClick={() => setExpanded(!isExpanded)}
          >
            <span className={cn("transition-transform", isExpanded ? "rotate-90" : "rotate-0")}>▸</span>
          </button>
        ) : (
          <span className="inline-block size-6" aria-hidden />
        )}

        <button
          type="button"
          role="treeitem"
          aria-selected={isSelected}
          aria-expanded={isExpandable ? isExpanded : undefined}
          data-value={value}
          className={cn(
            "flex min-w-0 flex-1 items-center gap-2 rounded-md px-2 py-1.5 text-left text-sm text-zinc-700 transition-colors hover:bg-emerald-50 hover:text-zinc-900",
            isSelected && "bg-emerald-100 text-emerald-900",
          )}
          tabIndex={isSelected ? 0 : -1}
          onClick={() => context.onSelectedValueChange?.(value)}
          onKeyDown={(event) => {
            if (event.key === "ArrowRight") {
              if (isExpandable && !isExpanded) {
                event.preventDefault();
                setExpanded(true);
              }
              return;
            }

            if (event.key === "ArrowLeft") {
              if (isExpandable && isExpanded) {
                event.preventDefault();
                setExpanded(false);
              }
            }
          }}
        >
          {icon ? <span className="inline-flex size-4 items-center justify-center">{icon}</span> : null}
          <span className="truncate">{label}</span>
        </button>
      </div>

      {hasNestedItems ? (
        <TreeViewGroup className={cn("ml-6 mt-1", !isExpanded && "hidden")}>{children}</TreeViewGroup>
      ) : null}
    </li>
  );
}

export function TreeViewGroup({
  className,
  ...props
}: React.HTMLAttributes<HTMLUListElement>) {
  return <ul role="group" className={cn("space-y-1", className)} {...props} />;
}

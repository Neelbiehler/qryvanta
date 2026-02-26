"use client";

import * as React from "react";

import { cn } from "../lib/cn";

export type SplitPaneDirection = "horizontal" | "vertical";

export type SplitPaneProps = React.HTMLAttributes<HTMLDivElement> & {
  direction?: SplitPaneDirection;
  defaultSizes?: number[];
  handleSize?: number;
};

export type SplitPanePanelProps = React.HTMLAttributes<HTMLDivElement> & {
  minSize?: number;
  collapsible?: boolean;
};

export type SplitPaneHandleProps = React.ButtonHTMLAttributes<HTMLButtonElement> & {
  direction?: SplitPaneDirection;
};

function normalizeSizes(panelCount: number, sizes?: number[]) {
  if (!sizes || sizes.length !== panelCount) {
    return Array.from({ length: panelCount }, () => 100 / panelCount);
  }

  const total = sizes.reduce((sum, value) => sum + value, 0);
  if (total <= 0) {
    return Array.from({ length: panelCount }, () => 100 / panelCount);
  }

  return sizes.map((value) => (value / total) * 100);
}

export function SplitPane({
  className,
  children,
  direction = "horizontal",
  defaultSizes,
  handleSize = 8,
  ...props
}: SplitPaneProps) {
  const childArray = React.Children.toArray(children);
  const panelElements = childArray.filter(
    (child): child is React.ReactElement<SplitPanePanelProps> =>
      React.isValidElement(child) && child.type === SplitPanePanel,
  );

  const panelCount = panelElements.length;
  const panelMeta = panelElements.map((panel) => ({
    minSize: panel.props.minSize ?? 12,
    collapsible: panel.props.collapsible ?? false,
  }));

  const [sizes, setSizes] = React.useState<number[]>(() => normalizeSizes(panelCount, defaultSizes));
  const containerRef = React.useRef<HTMLDivElement | null>(null);
  const dragRef = React.useRef<{
    handleIndex: number;
    startPosition: number;
    startSizes: number[];
  } | null>(null);

  React.useEffect(() => {
    setSizes((current) => normalizeSizes(panelCount, current.length === panelCount ? current : defaultSizes));
  }, [defaultSizes, panelCount]);

  React.useEffect(() => {
    function handlePointerMove(event: PointerEvent) {
      const drag = dragRef.current;
      const container = containerRef.current;

      if (!drag || !container) {
        return;
      }

      const rect = container.getBoundingClientRect();
      const totalSize = direction === "horizontal" ? rect.width : rect.height;

      if (totalSize <= 0) {
        return;
      }

      const pointerPosition = direction === "horizontal" ? event.clientX : event.clientY;
      const deltaPercent = ((pointerPosition - drag.startPosition) / totalSize) * 100;

      const leftIndex = drag.handleIndex;
      const rightIndex = drag.handleIndex + 1;
      const combinedSize = drag.startSizes[leftIndex] + drag.startSizes[rightIndex];

      const leftMeta = panelMeta[leftIndex];
      const rightMeta = panelMeta[rightIndex];

      const leftMin = leftMeta.collapsible ? 0 : leftMeta.minSize;
      const rightMin = rightMeta.collapsible ? 0 : rightMeta.minSize;

      let nextLeftSize = Math.max(leftMin, Math.min(drag.startSizes[leftIndex] + deltaPercent, combinedSize - rightMin));
      let nextRightSize = combinedSize - nextLeftSize;

      if (leftMeta.collapsible && nextLeftSize < leftMeta.minSize / 2) {
        nextLeftSize = 0;
        nextRightSize = combinedSize;
      }

      if (rightMeta.collapsible && nextRightSize < rightMeta.minSize / 2) {
        nextRightSize = 0;
        nextLeftSize = combinedSize;
      }

      setSizes((current) => {
        const next = [...current];
        next[leftIndex] = nextLeftSize;
        next[rightIndex] = nextRightSize;
        return next;
      });
    }

    function handlePointerUp() {
      dragRef.current = null;
    }

    window.addEventListener("pointermove", handlePointerMove);
    window.addEventListener("pointerup", handlePointerUp);

    return () => {
      window.removeEventListener("pointermove", handlePointerMove);
      window.removeEventListener("pointerup", handlePointerUp);
    };
  }, [direction, panelMeta]);

  let panelIndex = 0;
  let handleIndex = 0;
  const templateParts: string[] = [];

  const renderedChildren = childArray.map((child) => {
    if (!React.isValidElement(child)) {
      return child;
    }

    if (React.isValidElement<SplitPanePanelProps>(child) && child.type === SplitPanePanel) {
      const resolvedSize = sizes[panelIndex] ?? 100 / Math.max(panelCount, 1);
      templateParts.push(`${resolvedSize}%`);

      const currentPanelIndex = panelIndex;
      panelIndex += 1;

      return React.cloneElement(child, {
        className: cn("min-h-0 min-w-0 overflow-auto", child.props.className),
        key: child.key ?? `panel-${currentPanelIndex}`,
      });
    }

    if (React.isValidElement<SplitPaneHandleProps>(child) && child.type === SplitPaneHandle) {
      templateParts.push(`${handleSize}px`);

      const currentHandleIndex = handleIndex;
      handleIndex += 1;

      return React.cloneElement(child, {
        direction,
        key: child.key ?? `handle-${currentHandleIndex}`,
        onPointerDown: (event: React.PointerEvent<HTMLButtonElement>) => {
          child.props.onPointerDown?.(event);
          if (event.defaultPrevented) {
            return;
          }

          dragRef.current = {
            handleIndex: currentHandleIndex,
            startPosition: direction === "horizontal" ? event.clientX : event.clientY,
            startSizes: [...sizes],
          };
        },
      });
    }

    return child;
  });

  const gridStyle =
    direction === "horizontal"
      ? { gridTemplateColumns: templateParts.join(" ") }
      : { gridTemplateRows: templateParts.join(" ") };

  return (
    <div
      ref={containerRef}
      className={cn(
        "grid h-full w-full min-h-0 min-w-0",
        direction === "horizontal" ? "grid-flow-col" : "grid-flow-row",
        className,
      )}
      style={gridStyle}
      {...props}
    >
      {renderedChildren}
    </div>
  );
}

export const SplitPanePanel = React.forwardRef<HTMLDivElement, SplitPanePanelProps>(
  ({ className, ...props }, ref) => <div ref={ref} className={cn("h-full", className)} {...props} />,
);

SplitPanePanel.displayName = "SplitPanePanel";

const SplitPaneHandleImpl = React.forwardRef<HTMLButtonElement, SplitPaneHandleProps>(
  ({ className, direction = "horizontal", ...props }, ref) => (
    <button
      ref={ref}
      type="button"
      aria-label="Resize panels"
      className={cn(
        "group relative border-0 bg-[var(--split-handle-bg,#d8ebdf)] p-0 transition-colors hover:bg-[var(--split-handle-hover,#9dc9b4)] active:bg-[var(--split-handle-active,#2f8f63)]",
        direction === "horizontal" ? "h-full cursor-col-resize" : "w-full cursor-row-resize",
        className,
      )}
      {...props}
    >
      <span
        aria-hidden
        className={cn(
          "absolute rounded-sm bg-white/70",
          direction === "horizontal"
            ? "left-1/2 top-1/2 h-8 w-1 -translate-x-1/2 -translate-y-1/2"
            : "left-1/2 top-1/2 h-1 w-8 -translate-x-1/2 -translate-y-1/2",
        )}
      />
    </button>
  ),
);

SplitPaneHandleImpl.displayName = "SplitPaneHandle";

export const SplitPaneHandle = SplitPaneHandleImpl;

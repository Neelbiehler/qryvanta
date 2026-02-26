import * as React from "react";

import { cn } from "../lib/cn";

type AccordionContextValue = {
  openValues: string[];
  toggleItem: (value: string) => void;
};

const AccordionContext = React.createContext<AccordionContextValue | null>(null);

function useAccordionContext() {
  const context = React.useContext(AccordionContext);
  if (!context) {
    throw new Error("Accordion components must be wrapped in Accordion");
  }

  return context;
}

type AccordionItemContextValue = {
  value: string;
  open: boolean;
};

const AccordionItemContext = React.createContext<AccordionItemContextValue | null>(null);

function useAccordionItemContext() {
  const context = React.useContext(AccordionItemContext);
  if (!context) {
    throw new Error("AccordionTrigger and AccordionContent must be wrapped in AccordionItem");
  }

  return context;
}

type AccordionSingleProps = {
  type: "single";
  value: string | null;
  onValueChange: (value: string | null) => void;
};

type AccordionMultipleProps = {
  type: "multiple";
  value: string[];
  onValueChange: (value: string[]) => void;
};

export type AccordionProps = React.HTMLAttributes<HTMLDivElement> &
  (AccordionSingleProps | AccordionMultipleProps);

export function Accordion({ className, children, ...props }: AccordionProps) {
  const openValues =
    props.type === "single" ? (props.value ? [props.value] : []) : (props.value ?? []);

  function toggleItem(value: string) {
    if (props.type === "single") {
      const isOpen = props.value === value;
      props.onValueChange(isOpen ? null : value);
      return;
    }

    const current = props.value ?? [];
    if (current.includes(value)) {
      props.onValueChange(current.filter((entry) => entry !== value));
      return;
    }

    props.onValueChange([...current, value]);
  }

  return (
    <AccordionContext.Provider value={{ openValues, toggleItem }}>
      <div className={cn("w-full", className)}>{children}</div>
    </AccordionContext.Provider>
  );
}

export type AccordionItemProps = React.HTMLAttributes<HTMLDivElement> & {
  value: string;
};

export function AccordionItem({ className, value, ...props }: AccordionItemProps) {
  const { openValues } = useAccordionContext();
  const open = openValues.includes(value);

  return (
    <AccordionItemContext.Provider value={{ value, open }}>
      <div
        data-state={open ? "open" : "closed"}
        className={cn("border-b border-emerald-100", className)}
        {...props}
      />
    </AccordionItemContext.Provider>
  );
}

export function AccordionTrigger({
  className,
  children,
  onClick,
  ...props
}: React.ButtonHTMLAttributes<HTMLButtonElement>) {
  const { toggleItem } = useAccordionContext();
  const { value, open } = useAccordionItemContext();

  return (
    <button
      type="button"
      aria-expanded={open}
      className={cn(
        "flex w-full items-center justify-between gap-3 py-3 text-left text-sm font-medium text-zinc-700 transition-colors hover:text-zinc-900",
        className,
      )}
      onClick={(event) => {
        onClick?.(event);
        if (!event.defaultPrevented) {
          toggleItem(value);
        }
      }}
      {...props}
    >
      <span>{children}</span>
      <span
        aria-hidden
        className={cn(
          "inline-flex size-4 items-center justify-center text-zinc-500 transition-transform",
          open ? "rotate-180" : "rotate-0",
        )}
      >
        ▾
      </span>
    </button>
  );
}

export function AccordionContent({
  className,
  children,
  ...props
}: React.HTMLAttributes<HTMLDivElement>) {
  const { open } = useAccordionItemContext();

  return (
    <div
      data-state={open ? "open" : "closed"}
      className={cn(
        "grid overflow-hidden text-sm text-zinc-600 transition-[grid-template-rows,opacity] duration-200",
        open ? "grid-rows-[1fr] opacity-100" : "grid-rows-[0fr] opacity-0",
        className,
      )}
      {...props}
    >
      <div className="min-h-0 pb-3">{children}</div>
    </div>
  );
}

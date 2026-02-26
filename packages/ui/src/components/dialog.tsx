"use client";

import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { createPortal } from "react-dom";

import { cn } from "../lib/cn";

type DialogContextValue = {
  contentId: string;
  titleId: string;
  descriptionId: string;
  open: boolean;
  setOpen: (nextOpen: boolean) => void;
};

const DialogContext = React.createContext<DialogContextValue | null>(null);

function useDialogContext() {
  const context = React.useContext(DialogContext);
  if (!context) {
    throw new Error("Dialog components must be wrapped in Dialog");
  }

  return context;
}

function useControllableOpenState({
  open,
  defaultOpen,
  onOpenChange,
}: {
  open?: boolean;
  defaultOpen?: boolean;
  onOpenChange?: (open: boolean) => void;
}) {
  const [internalOpen, setInternalOpen] = React.useState(defaultOpen ?? false);
  const isControlled = open !== undefined;
  const value = isControlled ? open : internalOpen;

  const setValue = React.useCallback(
    (nextOpen: boolean) => {
      if (!isControlled) {
        setInternalOpen(nextOpen);
      }
      onOpenChange?.(nextOpen);
    },
    [isControlled, onOpenChange],
  );

  return [value, setValue] as const;
}

function getFocusableElements(container: HTMLElement) {
  return Array.from(
    container.querySelectorAll<HTMLElement>(
      'a[href], button:not([disabled]), textarea:not([disabled]), input:not([disabled]), select:not([disabled]), [tabindex]:not([tabindex="-1"])',
    ),
  ).filter((element) => !element.hasAttribute("disabled") && element.tabIndex >= 0);
}

function mergeRefs<T>(targetRef: React.ForwardedRef<T>, node: T | null) {
  if (typeof targetRef === "function") {
    targetRef(node);
    return;
  }

  if (targetRef) {
    targetRef.current = node;
  }
}

export type DialogProps = {
  children: React.ReactNode;
  open?: boolean;
  defaultOpen?: boolean;
  onOpenChange?: (open: boolean) => void;
};

export function Dialog({ children, open, defaultOpen, onOpenChange }: DialogProps) {
  const [isOpen, setOpen] = useControllableOpenState({
    open,
    defaultOpen,
    onOpenChange,
  });

  const contentId = React.useId();
  const titleId = React.useId();
  const descriptionId = React.useId();

  return (
    <DialogContext.Provider
      value={{
        contentId,
        titleId,
        descriptionId,
        open: isOpen,
        setOpen,
      }}
    >
      {children}
    </DialogContext.Provider>
  );
}

export function DialogTrigger({
  children,
}: {
  children: React.ReactNode;
}) {
  const { open, setOpen } = useDialogContext();

  if (!React.isValidElement<React.HTMLAttributes<HTMLElement>>(children)) {
    return <>{children}</>;
  }

  const childProps = children.props;
  return React.cloneElement(children, {
    "aria-expanded": open,
    "aria-haspopup": "dialog",
    onClick: (event: React.MouseEvent<HTMLElement>) => {
      childProps.onClick?.(event);
      if (!event.defaultPrevented) {
        setOpen(!open);
      }
    },
  });
}

export function DialogClose({
  children,
}: {
  children: React.ReactNode;
}) {
  const { setOpen } = useDialogContext();

  if (!React.isValidElement<React.HTMLAttributes<HTMLElement>>(children)) {
    return <>{children}</>;
  }

  const childProps = children.props;
  return React.cloneElement(children, {
    onClick: (event: React.MouseEvent<HTMLElement>) => {
      childProps.onClick?.(event);
      if (!event.defaultPrevented) {
        setOpen(false);
      }
    },
  });
}

const dialogContentVariants = cva(
  "relative w-full rounded-lg border border-emerald-200 bg-white p-5 text-zinc-900 shadow-xl focus:outline-none",
  {
    variants: {
      size: {
        sm: "max-w-lg",
        md: "max-w-2xl",
        lg: "max-w-4xl",
        xl: "max-w-6xl",
      },
    },
    defaultVariants: {
      size: "md",
    },
  },
);

export type DialogContentProps = React.HTMLAttributes<HTMLDivElement> &
  VariantProps<typeof dialogContentVariants> & {
    showCloseButton?: boolean;
  };

export const DialogContent = React.forwardRef<HTMLDivElement, DialogContentProps>(
  ({ className, size, children, showCloseButton = true, ...props }, ref) => {
    const { contentId, titleId, descriptionId, open, setOpen } = useDialogContext();
    const [mounted, setMounted] = React.useState(false);
    const contentRef = React.useRef<HTMLDivElement | null>(null);

    React.useEffect(() => {
      setMounted(true);
    }, []);

    React.useEffect(() => {
      if (!open) {
        return;
      }

      const previousFocusedElement = document.activeElement as HTMLElement | null;
      const container = contentRef.current;

      if (container) {
        const focusableElements = getFocusableElements(container);
        const nextFocus = focusableElements[0] ?? container;
        nextFocus.focus();
      }

      function handleKeyDown(event: KeyboardEvent) {
        if (!contentRef.current) {
          return;
        }

        if (event.key === "Escape") {
          event.preventDefault();
          setOpen(false);
          return;
        }

        if (event.key !== "Tab") {
          return;
        }

        const focusableElements = getFocusableElements(contentRef.current);
        if (focusableElements.length === 0) {
          event.preventDefault();
          contentRef.current.focus();
          return;
        }

        const firstElement = focusableElements[0];
        const lastElement = focusableElements[focusableElements.length - 1];
        const activeElement = document.activeElement as HTMLElement | null;

        if (event.shiftKey && activeElement === firstElement) {
          event.preventDefault();
          lastElement.focus();
          return;
        }

        if (!event.shiftKey && activeElement === lastElement) {
          event.preventDefault();
          firstElement.focus();
        }
      }

      document.addEventListener("keydown", handleKeyDown);
      return () => {
        document.removeEventListener("keydown", handleKeyDown);
        previousFocusedElement?.focus();
      };
    }, [open, setOpen]);

    if (!mounted || !open) {
      return null;
    }

    return createPortal(
      <div className="fixed inset-0 z-50" role="presentation">
        <div
          className="absolute inset-0 bg-zinc-950/45"
          onMouseDown={() => setOpen(false)}
          role="presentation"
        />
        <div className="relative flex min-h-full items-center justify-center p-4 md:p-6">
          <div
            ref={(node) => {
              contentRef.current = node;
              mergeRefs(ref, node);
            }}
            role="dialog"
            aria-modal="true"
            aria-labelledby={titleId}
            aria-describedby={descriptionId}
            id={contentId}
            tabIndex={-1}
            className={cn(dialogContentVariants({ size }), className)}
            onMouseDown={(event) => event.stopPropagation()}
            {...props}
          >
            {showCloseButton ? (
              <button
                type="button"
                className="absolute right-3 top-3 inline-flex size-7 items-center justify-center rounded-md text-zinc-500 transition-colors hover:bg-zinc-100 hover:text-zinc-700"
                onClick={() => setOpen(false)}
                aria-label="Close dialog"
              >
                ×
              </button>
            ) : null}
            {children}
          </div>
        </div>
      </div>,
      document.body,
    );
  },
);

DialogContent.displayName = "DialogContent";

export function DialogHeader({
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement>) {
  return <div className={cn("mb-4 space-y-1.5 pr-8", className)} {...props} />;
}

export const DialogTitle = React.forwardRef<HTMLHeadingElement, React.HTMLAttributes<HTMLHeadingElement>>(
  ({ className, ...props }, ref) => {
    const { titleId } = useDialogContext();

    return (
      <h2 id={titleId} ref={ref} className={cn("text-lg font-semibold", className)} {...props} />
    );
  },
);

DialogTitle.displayName = "DialogTitle";

export const DialogDescription = React.forwardRef<
  HTMLParagraphElement,
  React.HTMLAttributes<HTMLParagraphElement>
>(({ className, ...props }, ref) => {
  const { descriptionId } = useDialogContext();

  return (
    <p
      id={descriptionId}
      ref={ref}
      className={cn("text-sm text-zinc-600", className)}
      {...props}
    />
  );
});

DialogDescription.displayName = "DialogDescription";

export function DialogFooter({
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      className={cn("mt-5 flex flex-col-reverse gap-2 sm:flex-row sm:justify-end", className)}
      {...props}
    />
  );
}

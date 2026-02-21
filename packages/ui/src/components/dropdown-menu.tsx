"use client";

import * as React from "react";

import { cn } from "../lib/cn";

type DropdownMenuContextValue = {
  menuId: string;
  open: boolean;
  setOpen: React.Dispatch<React.SetStateAction<boolean>>;
};

const DropdownMenuContext = React.createContext<DropdownMenuContextValue | null>(
  null,
);

function useDropdownMenuContext() {
  const context = React.useContext(DropdownMenuContext);
  if (!context) {
    throw new Error("Dropdown menu components must be wrapped in DropdownMenu");
  }

  return context;
}

export function DropdownMenu({ children }: { children: React.ReactNode }) {
  const [open, setOpen] = React.useState(false);
  const menuId = React.useId();
  const containerRef = React.useRef<HTMLDivElement | null>(null);

  React.useEffect(() => {
    function handlePointerDown(event: MouseEvent) {
      const container = containerRef.current;
      if (!container) {
        return;
      }

      if (!container.contains(event.target as Node)) {
        setOpen(false);
      }
    }

    document.addEventListener("mousedown", handlePointerDown);
    return () => {
      document.removeEventListener("mousedown", handlePointerDown);
    };
  }, []);

  return (
    <DropdownMenuContext.Provider value={{ menuId, open, setOpen }}>
      <div className="relative inline-block text-left" ref={containerRef}>
        {children}
      </div>
    </DropdownMenuContext.Provider>
  );
}

export function DropdownMenuTrigger({
  children,
}: {
  children: React.ReactNode;
}) {
  const { menuId, open, setOpen } = useDropdownMenuContext();

  if (!React.isValidElement<React.HTMLAttributes<HTMLElement>>(children)) {
    return <>{children}</>;
  }

  const childProps = children.props;
  return React.cloneElement(children, {
    "aria-controls": menuId,
    "aria-expanded": open,
    "aria-haspopup": "menu",
    onClick: (event: React.MouseEvent<HTMLElement>) => {
      childProps.onClick?.(event);
      if (event.defaultPrevented) {
        return;
      }
      setOpen((current) => !current);
    },
  });
}

export function DropdownMenuContent({
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement>) {
  const { menuId, open } = useDropdownMenuContext();

  if (!open) {
    return null;
  }

  return (
    <div
      id={menuId}
      role="menu"
      className={cn(
        "absolute right-0 z-10 mt-2 w-56 rounded-md border border-emerald-200 bg-white p-1 shadow-lg",
        className,
      )}
      {...props}
    />
  );
}

export function DropdownMenuItem({
  className,
  onClick,
  ...props
}: React.ButtonHTMLAttributes<HTMLButtonElement>) {
  const { setOpen } = useDropdownMenuContext();

  return (
    <button
      type="button"
      role="menuitem"
      className={cn(
        "flex w-full items-center rounded-sm px-2 py-1.5 text-sm hover:bg-emerald-100",
        className,
      )}
      onClick={(event) => {
        onClick?.(event);
        if (!event.defaultPrevented) {
          setOpen(false);
        }
      }}
      {...props}
    />
  );
}

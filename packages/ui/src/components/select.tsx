import * as React from "react";

import { cn } from "../lib/cn";

const Select = React.forwardRef<HTMLSelectElement, React.ComponentProps<"select">>(
  ({ className, ...props }, ref) => (
    <select
      ref={ref}
      className={cn(
        "h-10 w-full rounded-md border border-emerald-200 bg-white px-3 text-sm text-zinc-900 shadow-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-emerald-500",
        className,
      )}
      {...props}
    />
  ),
);

Select.displayName = "Select";

export { Select };

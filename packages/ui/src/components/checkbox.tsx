import * as React from "react";

import { cn } from "../lib/cn";

const Checkbox = React.forwardRef<
  HTMLInputElement,
  Omit<React.ComponentProps<"input">, "type">
>(({ className, ...props }, ref) => (
  <input
    ref={ref}
    type="checkbox"
    className={cn(
      "h-4 w-4 rounded border-emerald-300 text-emerald-700 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-emerald-500",
      className,
    )}
    {...props}
  />
));

Checkbox.displayName = "Checkbox";

export { Checkbox };

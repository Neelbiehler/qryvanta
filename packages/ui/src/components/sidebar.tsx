import * as React from "react";

import { cn } from "../lib/cn";

export function Sidebar({
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement>) {
  return (
    <aside
      className={cn(
        "w-full border-r border-emerald-100 bg-gradient-to-b from-emerald-50 to-lime-50",
        className,
      )}
      {...props}
    />
  );
}

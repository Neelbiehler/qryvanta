import * as React from "react";

import { cn } from "../lib/cn";

export interface SidebarProps extends React.HTMLAttributes<HTMLDivElement> {
  collapsed?: boolean;
}

export function Sidebar({
  className,
  collapsed,
  ...props
}: SidebarProps) {
  return (
    <aside
      className={cn(
        "flex h-full flex-col border-r border-emerald-200/60 bg-white/80 backdrop-blur-sm",
        collapsed ? "w-16" : "w-full",
        className,
      )}
      {...props}
    />
  );
}

import * as React from "react";

import { Input } from "./input";
import { cn } from "../lib/cn";

export type SearchFilterBarProps = React.HTMLAttributes<HTMLDivElement> & {
  searchValue: string;
  onSearchValueChange: (value: string) => void;
  searchPlaceholder?: string;
  filters?: React.ReactNode;
  actions?: React.ReactNode;
};

export function SearchFilterBar({
  className,
  searchValue,
  onSearchValueChange,
  searchPlaceholder = "Search",
  filters,
  actions,
  ...props
}: SearchFilterBarProps) {
  return (
    <div
      className={cn(
        "flex flex-col gap-2 rounded-md border border-emerald-200 bg-white p-3 md:flex-row md:items-center",
        className,
      )}
      {...props}
    >
      <Input
        value={searchValue}
        onChange={(event) => onSearchValueChange(event.currentTarget.value)}
        placeholder={searchPlaceholder}
        className="md:max-w-sm"
      />
      {filters ? <div className="flex flex-1 items-center gap-2">{filters}</div> : <div className="flex-1" />}
      {actions ? <div className="flex items-center gap-2">{actions}</div> : null}
    </div>
  );
}

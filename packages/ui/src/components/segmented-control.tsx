import * as React from "react";

import { cn } from "../lib/cn";

export type SegmentedControlOption<T extends string = string> = {
  value: T;
  label: string;
  icon?: React.ReactNode;
};

type SegmentedControlProps<T extends string = string> = {
  options: SegmentedControlOption<T>[];
  value: T;
  onChange: (value: T) => void;
  size?: "sm" | "default";
  className?: string;
};

export function SegmentedControl<T extends string = string>({
  options,
  value,
  onChange,
  size = "default",
  className,
}: SegmentedControlProps<T>) {
  return (
    <div
      role="group"
      className={cn(
        "inline-flex items-center gap-0.5 rounded-md border border-emerald-100 bg-zinc-50 p-0.5",
        className,
      )}
    >
      {options.map((option) => {
        const isSelected = option.value === value;
        return (
          <button
            key={option.value}
            type="button"
            role="radio"
            aria-checked={isSelected}
            aria-label={option.label}
            onClick={() => onChange(option.value)}
            className={cn(
              "inline-flex items-center gap-1 rounded font-medium transition-colors motion-reduce:transition-none",
              size === "sm" ? "h-6 px-2 text-[11px]" : "h-7 px-2.5 text-xs",
              isSelected
                ? "bg-white text-emerald-700 shadow-sm"
                : "text-zinc-500 hover:text-zinc-800",
            )}
          >
            {option.icon !== undefined ? (
              <span aria-hidden="true">{option.icon}</span>
            ) : null}
            {option.label}
          </button>
        );
      })}
    </div>
  );
}

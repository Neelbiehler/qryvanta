"use client";

import type { FormSubgrid } from "@/components/studio/types";

type SubgridPreviewProps = {
  subgrid: FormSubgrid;
};

export function SubgridPreview({ subgrid }: SubgridPreviewProps) {
  return (
    <div className="rounded-md border border-dashed border-zinc-300 bg-zinc-100 p-2">
      <p className="text-[10px] font-semibold uppercase tracking-[0.14em] text-zinc-500">
        {subgrid.display_name || "Sub-grid"}
      </p>
      <div className="mt-1 overflow-hidden rounded border border-zinc-200 bg-white">
        <div className="grid grid-cols-3 border-b border-zinc-200 bg-zinc-50 px-2 py-1 text-[10px] font-semibold uppercase tracking-[0.12em] text-zinc-500">
          <span>{subgrid.columns[0] ?? "Column 1"}</span>
          <span>{subgrid.columns[1] ?? "Column 2"}</span>
          <span>{subgrid.columns[2] ?? "Column 3"}</span>
        </div>
        <div className="grid grid-cols-3 px-2 py-1 text-[11px] text-zinc-400">
          <span>sample...</span>
          <span>sample...</span>
          <span>sample...</span>
        </div>
      </div>
    </div>
  );
}

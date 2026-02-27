"use client";

import { Input } from "@qryvanta/ui";

import type { ViewEditorState } from "@/components/studio/hooks/use-view-editor-state";
import type { FieldResponse } from "@/lib/api";

type ColumnDragPaletteProps = {
  fields: FieldResponse[];
  viewEditor: ViewEditorState;
  search: string;
  onSearchChange: (value: string) => void;
};

export function ColumnDragPalette({
  fields,
  viewEditor,
  search,
  onSearchChange,
}: ColumnDragPaletteProps) {
  const query = search.trim().toLowerCase();
  const filtered = !query
    ? fields
    : fields.filter((field) =>
        `${field.display_name} ${field.logical_name} ${field.field_type}`
          .toLowerCase()
          .includes(query),
      );

  return (
    <aside className="rounded-lg border border-zinc-200 bg-white p-3">
      <p className="text-[10px] font-semibold uppercase tracking-[0.14em] text-zinc-500">
        Column Palette
      </p>
      <Input
        value={search}
        onChange={(event) => onSearchChange(event.target.value)}
        placeholder="Search fields"
        className="mt-2 h-8 text-xs"
      />
      <div className="mt-2 max-h-72 space-y-1 overflow-y-auto">
        {filtered.map((field) => (
          <button
            key={field.logical_name}
            type="button"
            draggable
            onDragStart={(event) => {
              event.dataTransfer.setData("text/view-column-field", field.logical_name);
            }}
            onClick={() => viewEditor.addColumn(field.logical_name)}
            className="flex w-full items-center gap-2 rounded-md border border-zinc-200 bg-zinc-50 px-2 py-1.5 text-left hover:border-emerald-300"
          >
            <span className="truncate text-xs text-zinc-700">{field.display_name}</span>
            <span className="ml-auto text-[10px] text-zinc-400">{field.field_type}</span>
          </button>
        ))}
      </div>
    </aside>
  );
}

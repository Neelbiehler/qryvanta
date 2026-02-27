"use client";

import { Button, Checkbox, Input, Label, Select } from "@qryvanta/ui";

import type { ViewEditorState } from "@/components/studio/hooks/use-view-editor-state";
import type { FieldResponse } from "@/lib/api";

type ViewPropertiesPanelProps = {
  viewEditor: ViewEditorState;
  fields: FieldResponse[];
  viewMeta: {
    logicalName: string;
    displayName: string;
    viewType: string;
    isDefault: boolean;
  } | null;
  onSetViewMeta: (patch: Partial<{ displayName: string; viewType: string; isDefault: boolean }>) => void;
};

export function ViewPropertiesPanel({
  viewEditor,
  fields,
  viewMeta,
  onSetViewMeta,
}: ViewPropertiesPanelProps) {
  const selectedColumn =
    viewEditor.selectedColumnIndex !== null
      ? viewEditor.columns[viewEditor.selectedColumnIndex] ?? null
      : null;

  return (
    <aside className="flex h-full min-h-0 flex-col overflow-y-auto rounded-xl border border-zinc-200 bg-white p-3">
      <p className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500">
        View Properties
      </p>

      {viewMeta ? (
        <div className="mt-3 space-y-2 border-b border-zinc-200 pb-3">
          <div className="space-y-1">
            <Label htmlFor="studio_view_display_name" className="text-[11px]">
              Display Name
            </Label>
            <Input
              id="studio_view_display_name"
              value={viewMeta.displayName}
              onChange={(event) => onSetViewMeta({ displayName: event.target.value })}
              className="h-8 text-sm"
            />
          </div>
          <div className="space-y-1">
            <Label htmlFor="studio_view_type" className="text-[11px]">
              View Type
            </Label>
            <Select
              id="studio_view_type"
              value={viewMeta.viewType}
              onChange={(event) => onSetViewMeta({ viewType: event.target.value })}
              className="h-8 text-sm"
            >
              <option value="grid">Grid</option>
              <option value="card">Card</option>
            </Select>
          </div>
          <div className="flex items-center gap-2">
            <Checkbox
              id="studio_view_default"
              checked={viewMeta.isDefault}
              onChange={(event) => onSetViewMeta({ isDefault: event.target.checked })}
            />
            <Label htmlFor="studio_view_default" className="text-xs">
              Default View
            </Label>
          </div>
        </div>
      ) : null}

      <div className="mt-3 space-y-2">
        <p className="text-[10px] font-semibold uppercase tracking-[0.14em] text-zinc-400">
          Sort
        </p>
        <Select
          value={viewEditor.defaultSort?.field_logical_name ?? ""}
          onChange={(event) => {
            const field = event.target.value;
            if (!field) {
              viewEditor.setDefaultSort(null);
              return;
            }
            viewEditor.setDefaultSort({
              field_logical_name: field,
              direction: viewEditor.defaultSort?.direction ?? "asc",
            });
          }}
          className="h-8 text-xs"
        >
          <option value="">None</option>
          {fields.map((field) => (
            <option key={field.logical_name} value={field.logical_name}>
              {field.display_name}
            </option>
          ))}
        </Select>
        <Select
          value={viewEditor.defaultSort?.direction ?? "asc"}
          onChange={(event) =>
            viewEditor.setDefaultSort(
              viewEditor.defaultSort
                ? {
                    ...viewEditor.defaultSort,
                    direction: event.target.value === "desc" ? "desc" : "asc",
                  }
                : null,
            )
          }
          disabled={!viewEditor.defaultSort}
          className="h-8 text-xs"
        >
          <option value="asc">Ascending</option>
          <option value="desc">Descending</option>
        </Select>
      </div>

      <div className="mt-3 space-y-2 border-t border-zinc-200 pt-3">
        <p className="text-[10px] font-semibold uppercase tracking-[0.14em] text-zinc-400">
          Column Inspector
        </p>
        {selectedColumn ? (
          <>
            <Input
              value={selectedColumn.field_logical_name}
              onChange={(event) =>
                viewEditor.updateColumn(viewEditor.selectedColumnIndex!, {
                  field_logical_name: event.target.value,
                })
              }
              className="h-8 text-xs"
            />
            <Input
              value={selectedColumn.label_override ?? ""}
              onChange={(event) =>
                viewEditor.updateColumn(viewEditor.selectedColumnIndex!, {
                  label_override: event.target.value.trim() || null,
                })
              }
              placeholder="Label override"
              className="h-8 text-xs"
            />
            <Input
              type="number"
              min={60}
              value={selectedColumn.width ?? ""}
              onChange={(event) =>
                viewEditor.updateColumn(viewEditor.selectedColumnIndex!, {
                  width: event.target.value.trim() ? Number.parseInt(event.target.value, 10) : null,
                })
              }
              placeholder="Width"
              className="h-8 text-xs"
            />
            <div className="flex gap-1">
              <Button
                type="button"
                size="sm"
                variant="outline"
                onClick={() =>
                  viewEditor.reorderColumn(
                    viewEditor.selectedColumnIndex!,
                    Math.max(0, viewEditor.selectedColumnIndex! - 1),
                  )
                }
              >
                Left
              </Button>
              <Button
                type="button"
                size="sm"
                variant="outline"
                onClick={() =>
                  viewEditor.reorderColumn(
                    viewEditor.selectedColumnIndex!,
                    Math.min(viewEditor.columns.length - 1, viewEditor.selectedColumnIndex! + 1),
                  )
                }
              >
                Right
              </Button>
              <Button
                type="button"
                size="sm"
                variant="ghost"
                onClick={() => viewEditor.removeColumn(viewEditor.selectedColumnIndex!)}
              >
                Remove
              </Button>
            </div>
          </>
        ) : (
          <p className="text-xs text-zinc-500">Click a table header to edit a column.</p>
        )}
      </div>
    </aside>
  );
}

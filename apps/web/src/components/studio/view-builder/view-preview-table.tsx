"use client";

import type { ViewEditorState } from "@/components/studio/hooks/use-view-editor-state";
import type { FieldResponse, RuntimeRecordResponse } from "@/lib/api";

type ViewPreviewTableProps = {
  fields: FieldResponse[];
  viewEditor: ViewEditorState;
  onSelectRecord: (record: RuntimeRecordResponse) => void;
};

export function ViewPreviewTable({
  fields,
  viewEditor,
  onSelectRecord,
}: ViewPreviewTableProps) {
  if (viewEditor.columns.length === 0) {
    return <p className="text-sm text-zinc-500">Add columns to render preview table.</p>;
  }

  if (viewEditor.previewRows.length === 0) {
    return <p className="text-sm text-zinc-500">No preview rows for current filters.</p>;
  }

  return (
    <div className="overflow-x-auto rounded-lg border border-zinc-200 bg-white">
      <table className="w-full text-sm">
        <thead className="bg-zinc-50">
          <tr>
            {viewEditor.columns.map((column, index) => {
              const field = fields.find((candidate) => candidate.logical_name === column.field_logical_name);
              return (
                <th
                  key={`preview-head-${column.field_logical_name}`}
                  className="cursor-pointer border-b border-zinc-200 px-3 py-2 text-left text-xs font-semibold uppercase tracking-[0.12em] text-zinc-600"
                  onClick={() => viewEditor.setSelectedColumnIndex(index)}
                  style={column.width ? { width: `${column.width}px` } : undefined}
                  draggable
                  onDragStart={(event) => {
                    event.dataTransfer.setData("text/view-column-index", String(index));
                    event.dataTransfer.effectAllowed = "move";
                  }}
                  onDragOver={(event) => event.preventDefault()}
                  onDrop={(event) => {
                    const sourceIndex = Number.parseInt(
                      event.dataTransfer.getData("text/view-column-index"),
                      10,
                    );
                    if (Number.isNaN(sourceIndex) || sourceIndex === index) return;
                    viewEditor.reorderColumn(sourceIndex, index);
                  }}
                >
                  {column.label_override?.trim() || field?.display_name || column.field_logical_name}
                </th>
              );
            })}
          </tr>
        </thead>
        <tbody>
          {viewEditor.previewRows.map((record) => (
            <tr
              key={record.record_id}
              className="cursor-pointer border-b border-zinc-100 hover:bg-emerald-50/40"
              onClick={() => onSelectRecord(record)}
            >
              {viewEditor.columns.map((column) => (
                <td key={`${record.record_id}-${column.field_logical_name}`} className="px-3 py-2 align-top text-xs text-zinc-700">
                  {JSON.stringify((record.data as Record<string, unknown>)[column.field_logical_name] ?? null)}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

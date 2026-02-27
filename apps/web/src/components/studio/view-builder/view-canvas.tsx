"use client";

import { useState } from "react";

import { Button, StatusBadge } from "@qryvanta/ui";

import { ColumnDragPalette } from "@/components/studio/view-builder/column-drag-palette";
import { FilterBuilder } from "@/components/studio/view-builder/filter-builder";
import { QuickCreateSlideOver } from "@/components/studio/view-builder/quick-create-slide-over";
import { RecordSlideOver } from "@/components/studio/view-builder/record-slide-over";
import { ViewPreviewTable } from "@/components/studio/view-builder/view-preview-table";
import type { ViewEditorState } from "@/components/studio/hooks/use-view-editor-state";
import type { FieldResponse, RuntimeRecordResponse } from "@/lib/api";

type ViewCanvasProps = {
  appLogicalName: string;
  entityLogicalName: string;
  fields: FieldResponse[];
  viewEditor: ViewEditorState;
  onRefreshPreviewRecords: () => Promise<void>;
};

export function ViewCanvas({
  appLogicalName,
  entityLogicalName,
  fields,
  viewEditor,
  onRefreshPreviewRecords,
}: ViewCanvasProps) {
  const [paletteSearch, setPaletteSearch] = useState("");
  const [selectedRecord, setSelectedRecord] = useState<RuntimeRecordResponse | null>(null);
  const [quickCreateOpen, setQuickCreateOpen] = useState(false);

  return (
    <div className="flex h-full min-h-0 flex-col gap-2 rounded-xl border border-zinc-200 bg-zinc-50 p-3">
      <div className="flex flex-wrap items-center gap-2">
        <StatusBadge tone="neutral">Columns {viewEditor.columns.length}</StatusBadge>
        <StatusBadge tone="neutral">Preview rows {viewEditor.previewRows.length}</StatusBadge>
        <div className="ml-auto">
          <Button type="button" size="sm" onClick={() => setQuickCreateOpen(true)}>
            New
          </Button>
        </div>
      </div>

      <div className="grid min-h-0 flex-1 gap-2 xl:grid-cols-[260px_minmax(0,1fr)]">
        <ColumnDragPalette
          fields={fields}
          viewEditor={viewEditor}
          search={paletteSearch}
          onSearchChange={setPaletteSearch}
        />

        <div
          className="space-y-2 overflow-y-auto rounded-lg border border-zinc-200 bg-white p-3"
          onDragOver={(event) => event.preventDefault()}
          onDrop={(event) => {
            event.preventDefault();
            const fieldLogicalName = event.dataTransfer.getData("text/view-column-field");
            if (!fieldLogicalName) return;
            viewEditor.addColumn(fieldLogicalName);
          }}
        >
          <FilterBuilder fields={fields} viewEditor={viewEditor} />
          <ViewPreviewTable
            fields={fields}
            viewEditor={viewEditor}
            onSelectRecord={setSelectedRecord}
          />
        </div>
      </div>

      <RecordSlideOver
        appLogicalName={appLogicalName}
        entityLogicalName={entityLogicalName}
        record={selectedRecord}
        onClose={() => setSelectedRecord(null)}
      />
      <QuickCreateSlideOver
        appLogicalName={appLogicalName}
        entityLogicalName={entityLogicalName}
        open={quickCreateOpen}
        onClose={() => setQuickCreateOpen(false)}
        onCreated={() => {
          void onRefreshPreviewRecords();
        }}
      />
    </div>
  );
}

import Link from "next/link";

import { Button, EmptyState } from "@qryvanta/ui";
import { DataGrid, type DataGridColumn } from "@qryvanta/ui/data-grid";

import { formatFieldValue, formatValue } from "@/components/apps/workspace-entity/helpers";
import type {
  WorkerViewMode,
  WorkerGridDensity,
} from "@/components/apps/workspace-entity/workspace-toolbar";
import type {
  AppEntityCapabilitiesResponse,
  FieldResponse,
  OptionSetResponse,
  RuntimeRecordResponse,
} from "@/lib/api";
import type {
  ViewColumn,
  ViewSort,
} from "@/components/apps/workspace-entity/metadata-types";

type MetadataGridProps = {
  appLogicalName: string;
  activeFormLogicalName: string | null;
  activeViewLogicalName: string | null;
  capabilities: AppEntityCapabilitiesResponse;
  columns: Array<ViewColumn & { field: FieldResponse | null }>;
  deletingRecordId: string | null;
  entityLogicalName: string;
  filteredRecords: RuntimeRecordResponse[];
  onDeleteRecord: (recordId: string) => void;
  optionSets: OptionSetResponse[];
  records: RuntimeRecordResponse[];
  viewMode: WorkerViewMode;
  density: WorkerGridDensity;
  defaultSort: ViewSort | null;
};

export function MetadataGrid({
  appLogicalName,
  activeFormLogicalName,
  activeViewLogicalName,
  capabilities,
  columns,
  deletingRecordId,
  entityLogicalName,
  filteredRecords,
  onDeleteRecord,
  optionSets,
  records,
  viewMode,
  density,
  defaultSort,
}: MetadataGridProps) {
  const queryParams = new URLSearchParams();
  if (activeFormLogicalName) {
    queryParams.set("form", activeFormLogicalName);
  }
  if (activeViewLogicalName) {
    queryParams.set("view", activeViewLogicalName);
  }
  const detailSuffix = queryParams.toString().length > 0 ? `?${queryParams.toString()}` : "";

  const gridColumns: DataGridColumn<RuntimeRecordResponse>[] = [
    {
      key: "record_id",
      header: "Record ID",
      width: 260,
      pin: "left",
      sortable: true,
      cell: (record) => (
        <Link
          href={`/worker/apps/${appLogicalName}/${entityLogicalName}/${record.record_id}${detailSuffix}`}
          className="font-mono text-xs text-emerald-700 underline-offset-2 hover:underline"
        >
          {record.record_id}
        </Link>
      ),
    },
  ];

  if (viewMode === "grid") {
    for (const viewColumn of columns) {
      gridColumns.push({
        key: viewColumn.field_logical_name,
        header: viewColumn.label_override ?? viewColumn.field?.display_name ?? viewColumn.field_logical_name,
        width: viewColumn.width ? `${String(viewColumn.width)}px` : undefined,
        sortable: true,
        cell: (record) => {
          const renderedValue = viewColumn.field
            ? formatFieldValue(record.data[viewColumn.field_logical_name], viewColumn.field, optionSets)
            : formatValue(record.data[viewColumn.field_logical_name]);

          return (
            <span className="block max-w-[220px] truncate" title={renderedValue}>
              {renderedValue}
            </span>
          );
        },
      });
    }
  }

  gridColumns.push({
    key: viewMode === "grid" ? "snapshot" : "data",
    header: viewMode === "grid" ? "Snapshot" : "Data",
    cell: (record) => {
      if (viewMode === "grid") {
        return `${String(Object.keys(record.data).length)} populated field(s)`;
      }

      return <span className="font-mono text-xs">{JSON.stringify(record.data)}</span>;
    },
  });

  gridColumns.push({
    key: "actions",
    header: "Actions",
    pin: "right",
    width: 148,
    cell: (record) =>
      capabilities.can_delete ? (
        <Button
          disabled={deletingRecordId === record.record_id}
          variant="outline"
          size="sm"
          type="button"
          onClick={() => onDeleteRecord(record.record_id)}
        >
          {deletingRecordId === record.record_id ? "Deleting…" : "Delete"}
        </Button>
      ) : (
        <span className="text-xs text-zinc-500">No delete access</span>
      ),
  });

  return (
    <DataGrid
      columns={gridColumns}
      rows={filteredRecords}
      getRowId={(record) => record.record_id}
      defaultSortState={
        defaultSort
          ? { key: defaultSort.field_logical_name, direction: defaultSort.direction }
          : null
      }
      className={
        density === "compact"
          ? "rounded-lg border border-emerald-100 bg-white text-xs shadow-sm [&_th]:h-9 [&_th]:bg-emerald-50 [&_th]:px-2 [&_th]:text-emerald-800 [&_td]:p-2"
          : "rounded-lg border border-emerald-100 bg-white shadow-sm [&_th]:bg-emerald-50 [&_th]:text-emerald-800"
      }
      emptyState={
        <EmptyState
          title={records.length > 0 ? "No matching records" : "No records yet"}
          description={
            records.length > 0
              ? "Adjust search or filters to find records."
              : "Create your first row from Quick Create to populate this view."
          }
        />
      }
    />
  );
}

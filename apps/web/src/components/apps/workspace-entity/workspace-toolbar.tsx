import { Button, Input, SegmentedControl, StatusBadge } from "@qryvanta/ui";
import { Grid3X3, LayoutList, Plus, RefreshCw, Search } from "lucide-react";
import type { ReactNode } from "react";

import type { AppEntityCapabilitiesResponse } from "@/lib/api";

export type WorkerViewMode = "grid" | "json";
export type WorkerGridDensity = "comfortable" | "compact";

type WorkspaceToolbarProps = {
  capabilities: AppEntityCapabilitiesResponse;
  filteredRecordCount: number;
  schemaVersion: number;
  onRefresh: () => void;
  onCreateNew: () => void;
  onSearchChange: (value: string) => void;
  onViewModeChange: (viewMode: WorkerViewMode) => void;
  onDensityChange: (density: WorkerGridDensity) => void;
  isRefreshingRecords: boolean;
  recordSearch: string;
  viewMode: WorkerViewMode;
  density: WorkerGridDensity;
};

const VIEW_OPTIONS: { value: WorkerViewMode; label: string; icon: ReactNode }[] = [
  {
    value: "grid" as WorkerViewMode,
    label: "Grid",
    icon: <Grid3X3 className="h-3 w-3" />,
  },
  {
    value: "json" as WorkerViewMode,
    label: "JSON",
    icon: <LayoutList className="h-3 w-3" />,
  },
];

const DENSITY_OPTIONS: { value: WorkerGridDensity; label: string }[] = [
  { value: "comfortable" as WorkerGridDensity, label: "Comfortable" },
  { value: "compact" as WorkerGridDensity, label: "Compact" },
];

export function WorkspaceToolbar({
  capabilities,
  filteredRecordCount,
  schemaVersion,
  onRefresh,
  onCreateNew,
  onSearchChange,
  onViewModeChange,
  onDensityChange,
  isRefreshingRecords,
  recordSearch,
  viewMode,
  density,
}: WorkspaceToolbarProps) {
  return (
    <div className="space-y-2">
      {/* Status badges */}
      <div className="flex flex-wrap items-center gap-1.5">
        <StatusBadge tone="neutral">Schema v{schemaVersion}</StatusBadge>
        <StatusBadge tone={capabilities.can_create ? "success" : "warning"}>
          {capabilities.can_create ? "Can create" : "Create blocked"}
        </StatusBadge>
        {capabilities.can_delete ? (
          <StatusBadge tone="warning">Can delete</StatusBadge>
        ) : null}
        {isRefreshingRecords ? (
          <StatusBadge tone="neutral">Refreshing…</StatusBadge>
        ) : null}
        <StatusBadge tone="info" dot>
          {filteredRecordCount} record{filteredRecordCount !== 1 ? "s" : ""}
        </StatusBadge>
      </div>

      {/* Command bar */}
      <div className="flex flex-wrap items-center gap-1 rounded-lg border border-emerald-100 bg-white px-2 py-1.5 shadow-sm xl:flex-nowrap">
        {/* Primary actions */}
        <div className="flex items-center gap-1">
          <Button
            type="button"
            variant="default"
            size="sm"
            onClick={onCreateNew}
            disabled={!capabilities.can_create}
            className="h-7 gap-1.5 px-2.5 text-xs"
          >
            <Plus aria-hidden="true" className="h-3.5 w-3.5" />
            New
          </Button>
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={onRefresh}
            className="h-7 gap-1.5 px-2.5 text-xs"
          >
            <RefreshCw
              aria-hidden="true"
              className={`h-3.5 w-3.5 ${isRefreshingRecords ? "animate-spin" : ""}`}
            />
            Refresh
          </Button>
        </div>

        {/* Separator */}
        <div aria-hidden="true" className="mx-1 h-5 w-px shrink-0 bg-emerald-100" />

        {/* View mode */}
        <SegmentedControl
          value={viewMode}
          onChange={onViewModeChange}
          size="sm"
          options={VIEW_OPTIONS}
        />

        {/* Separator */}
        <div aria-hidden="true" className="mx-1 h-5 w-px shrink-0 bg-emerald-100" />

        {/* Density */}
        <SegmentedControl
          value={density}
          onChange={onDensityChange}
          size="sm"
          options={DENSITY_OPTIONS}
        />

        {/* Search (right-aligned) */}
        <div className="flex w-full items-center gap-2 xl:ml-auto xl:w-auto">
          <div className="relative w-full xl:w-64">
            <Search
              aria-hidden="true"
              className="pointer-events-none absolute left-2.5 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-zinc-400"
            />
            <Input
              value={recordSearch}
              onChange={(event) => onSearchChange(event.currentTarget.value)}
              placeholder="Quick find records…"
              autoComplete="off"
              spellCheck={false}
              className="h-8 pl-8 text-xs"
            />
          </div>
        </div>
      </div>
    </div>
  );
}

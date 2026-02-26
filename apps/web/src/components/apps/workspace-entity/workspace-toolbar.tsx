import {
  CommandBar,
  CommandBarAction,
  CommandBarGroup,
  CommandBarSeparator,
  Label,
  SearchFilterBar,
  Select,
  StatusBadge,
} from "@qryvanta/ui";

import type {
  AppEntityCapabilitiesResponse,
} from "@/lib/api";
import type {
  ParsedFormResponse,
  ParsedViewResponse,
} from "@/components/apps/workspace-entity/metadata-types";

export type WorkerViewMode = "grid" | "json";

type WorkspaceToolbarProps = {
  capabilities: AppEntityCapabilitiesResponse;
  filteredRecordCount: number;
  schemaVersion: number;
  forms: ParsedFormResponse[];
  views: ParsedViewResponse[];
  activeFormLogicalName: string | null;
  activeViewLogicalName: string | null;
  onActiveFormChange: (name: string) => void;
  onActiveViewChange: (name: string) => void;
  onRefresh: () => void;
  onSearchChange: (value: string) => void;
  onToggleCreatePanel: () => void;
  onViewModeChange: (viewMode: WorkerViewMode) => void;
  isRefreshingRecords: boolean;
  recordSearch: string;
  showCreatePanel: boolean;
  viewMode: WorkerViewMode;
};

export function WorkspaceToolbar({
  capabilities,
  filteredRecordCount,
  schemaVersion,
  forms,
  views,
  activeFormLogicalName,
  activeViewLogicalName,
  onActiveFormChange,
  onActiveViewChange,
  onRefresh,
  onSearchChange,
  onToggleCreatePanel,
  onViewModeChange,
  isRefreshingRecords,
  recordSearch,
  showCreatePanel,
  viewMode,
}: WorkspaceToolbarProps) {
  return (
    <div className="space-y-3">
      <div className="flex flex-wrap items-center justify-between gap-2 border-b border-zinc-200 pb-2">
        <p className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500">
          Command Bar
        </p>
        <p className="text-xs text-zinc-500">Model-driven runtime view</p>
      </div>

      <div className="flex flex-wrap items-center gap-2">
        <StatusBadge tone="success">Schema v{schemaVersion}</StatusBadge>
        <StatusBadge tone={capabilities.can_create ? "success" : "warning"}>
          Create {capabilities.can_create ? "Enabled" : "Disabled"}
        </StatusBadge>
        <StatusBadge tone={capabilities.can_delete ? "warning" : "neutral"}>
          Delete {capabilities.can_delete ? "Enabled" : "Disabled"}
        </StatusBadge>
        {isRefreshingRecords ? <StatusBadge tone="neutral">Refreshing records</StatusBadge> : null}
        <StatusBadge tone="info" dot>
          Records {filteredRecordCount}
        </StatusBadge>
      </div>

      <CommandBar className="rounded-md border border-zinc-200 bg-white px-2">
        <CommandBarGroup>
          <CommandBarAction
            type="button"
            variant={showCreatePanel ? "primary" : "default"}
            onClick={onToggleCreatePanel}
          >
            {showCreatePanel ? "Hide Quick Create" : "Quick Create"}
          </CommandBarAction>
          <CommandBarAction type="button" variant="default" onClick={onRefresh}>
            Refresh
          </CommandBarAction>
        </CommandBarGroup>
        <CommandBarSeparator />
        <CommandBarGroup>
          <CommandBarAction
            type="button"
            variant={viewMode === "grid" ? "primary" : "default"}
            onClick={() => onViewModeChange("grid")}
          >
            Grid View
          </CommandBarAction>
          <CommandBarAction
            type="button"
            variant={viewMode === "json" ? "primary" : "default"}
            onClick={() => onViewModeChange("json")}
          >
            JSON View
          </CommandBarAction>
        </CommandBarGroup>
      </CommandBar>

      <SearchFilterBar
        searchValue={recordSearch}
        onSearchValueChange={onSearchChange}
        searchPlaceholder="Search by record id or field value"
        filters={
          <>
            {views.length > 1 ? (
              <div className="space-y-1">
                <Label htmlFor="view-selector">Active View</Label>
                <Select
                  id="view-selector"
                  value={activeViewLogicalName ?? ""}
                  onChange={(event) => onActiveViewChange(event.target.value)}
                >
                  {views.map((view) => (
                    <option key={view.logical_name} value={view.logical_name}>
                      {view.display_name}
                    </option>
                  ))}
                </Select>
              </div>
            ) : null}

            {forms.length > 1 ? (
              <div className="space-y-1">
                <Label htmlFor="form-selector">Active Form</Label>
                <Select
                  id="form-selector"
                  value={activeFormLogicalName ?? ""}
                  onChange={(event) => onActiveFormChange(event.target.value)}
                >
                  {forms.map((form) => (
                    <option key={form.logical_name} value={form.logical_name}>
                      {form.display_name} ({form.form_type})
                    </option>
                  ))}
                </Select>
              </div>
            ) : null}
          </>
        }
        actions={<p className="text-xs text-zinc-500">{filteredRecordCount} visible row(s)</p>}
      />
    </div>
  );
}

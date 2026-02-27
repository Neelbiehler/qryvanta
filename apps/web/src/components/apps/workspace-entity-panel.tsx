"use client";

import { useEffect, useMemo, useState } from "react";
import { useRouter } from "next/navigation";

import { Label, Notice, Select } from "@qryvanta/ui";

import {
  apiFetch,
  type AppEntityBindingResponse,
  type AppEntityCapabilitiesResponse,
  type FieldResponse,
  type OptionSetResponse,
  type PublishedSchemaResponse,
  type RuntimeRecordResponse,
} from "@/lib/api";
import {
  buildFieldMap,
  parseFormResponse,
  parseViewResponse,
} from "@/components/apps/workspace-entity/helpers";
import { MetadataGrid } from "@/components/apps/workspace-entity/metadata-grid";
import {
  WorkspaceToolbar,
  type WorkerGridDensity,
  type WorkerViewMode,
} from "@/components/apps/workspace-entity/workspace-toolbar";
import { useWorkspaceViewRecords } from "@/components/apps/workspace-entity/hooks/use-workspace-view-records";
import type {
  ParsedFormResponse,
  ParsedViewResponse,
  ViewColumn,
} from "@/components/apps/workspace-entity/metadata-types";

// ---------------------------------------------------------------------------
// Props
// ---------------------------------------------------------------------------

type WorkspaceEntityPanelProps = {
  appLogicalName: string;
  entityLogicalName: string;
  binding: AppEntityBindingResponse | null;
  schema: PublishedSchemaResponse;
  capabilities: AppEntityCapabilitiesResponse;
  records: RuntimeRecordResponse[];
  forms: ParsedFormResponse[];
  views: ParsedViewResponse[];
  initialFormLogicalName?: string | null;
  initialViewLogicalName?: string | null;
};

type PanelState = {
  errorMessage: string | null;
  statusMessage: string | null;
  deletingRecordId: string | null;
};

type ViewState = {
  recordSearch: string;
  viewMode: WorkerViewMode;
  activeFormLogicalName: string | null;
  activeViewLogicalName: string | null;
};

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export function WorkspaceEntityPanel({
  appLogicalName,
  entityLogicalName,
  binding,
  schema,
  capabilities,
  records,
  forms,
  views,
  initialFormLogicalName,
  initialViewLogicalName,
}: WorkspaceEntityPanelProps) {
  const router = useRouter();

  const fieldMap = useMemo(() => buildFieldMap(schema), [schema]);

  const defaultFormName =
    initialFormLogicalName ??
    binding?.default_form_logical_name ??
    forms.find((f) => f.form_type === "main")?.logical_name ??
    forms[0]?.logical_name ??
    null;
  const defaultViewName =
    initialViewLogicalName ??
    binding?.default_list_view_logical_name ??
    views.find((v) => v.is_default)?.logical_name ??
    views[0]?.logical_name ??
    null;

  const [panelState, setPanelState] = useState<PanelState>({
    errorMessage: null,
    statusMessage: null,
    deletingRecordId: null,
  });
  const [viewState, setViewState] = useState<ViewState>({
    recordSearch: "",
    viewMode: binding?.default_view_mode ?? "grid",
    activeFormLogicalName: defaultFormName,
    activeViewLogicalName: defaultViewName,
  });
  const [gridDensity, setGridDensity] = useState<WorkerGridDensity>("comfortable");

  useEffect(() => {
    try {
      const stored = localStorage.getItem(`worker_grid_density_${appLogicalName}`);
      if (!stored) return;
      const nextDensity: WorkerGridDensity = stored === "compact" ? "compact" : "comfortable";
      queueMicrotask(() => {
        setGridDensity(nextDensity);
      });
    } catch {
      // ignore storage errors
    }
  }, [appLogicalName]);
  const activeView = useMemo(
    () => views.find((v) => v.logical_name === viewState.activeViewLogicalName) ?? views[0] ?? null,
    [views, viewState.activeViewLogicalName],
  );

  // Resolve grid columns from active ViewDefinition
  const gridColumns = useMemo((): Array<ViewColumn & { field: FieldResponse | null }> => {
    if (!activeView) {
      // Fallback: first 5 non-json fields
      return schema.fields
        .filter((f) => f.field_type !== "json")
        .slice(0, 5)
        .map((field, index) => ({
          field_logical_name: field.logical_name,
          position: index,
          width: null,
          label_override: null,
          field,
        }));
    }

    return activeView.columns.map((col) => ({
      ...col,
      field: fieldMap.get(col.field_logical_name) ?? null,
    }));
  }, [activeView, schema.fields, fieldMap]);

  const { runtimeRecords, isRefreshingRecords, refreshErrorMessage } =
    useWorkspaceViewRecords({
      appLogicalName,
      entityLogicalName,
      activeView,
      records,
    });

  const filteredRecords = useMemo(() => {
    const normalizedQuery = viewState.recordSearch.trim().toLowerCase();
    if (!normalizedQuery) {
      return runtimeRecords;
    }

    return runtimeRecords.filter((record) => {
      if (record.record_id.toLowerCase().includes(normalizedQuery)) {
        return true;
      }

      return JSON.stringify(record.data)
        .toLowerCase()
        .includes(normalizedQuery);
    });
  }, [runtimeRecords, viewState.recordSearch]);

  function setErrorMessage(next: string | null) {
    setPanelState((current) => ({ ...current, errorMessage: next }));
  }

  function setStatusMessage(next: string | null) {
    setPanelState((current) => ({ ...current, statusMessage: next }));
  }

  function clearMessages() {
    setPanelState((current) => ({
      ...current,
      errorMessage: null,
      statusMessage: null,
    }));
  }

  async function handleDeleteRecord(recordId: string) {
    if (!capabilities.can_delete) {
      setErrorMessage(
        "You do not have delete permission for this entity in this app.",
      );
      return;
    }

    setPanelState((current) => ({ ...current, deletingRecordId: recordId }));
    clearMessages();

    try {
      const response = await apiFetch(
        `/api/workspace/apps/${appLogicalName}/entities/${entityLogicalName}/records/${recordId}`,
        {
          method: "DELETE",
        },
      );

      if (!response.ok) {
        const body = (await response.json()) as { message?: string };
        setErrorMessage(body.message ?? "Unable to delete record.");
        return;
      }

      setStatusMessage("Record deleted.");
      router.refresh();
    } catch {
      setErrorMessage("Unable to delete record.");
    } finally {
      setPanelState((current) => ({ ...current, deletingRecordId: null }));
    }
  }

  return (
    <WorkspaceEntityLayout
      appLogicalName={appLogicalName}
      entityLogicalName={entityLogicalName}
      capabilities={capabilities}
      schema={schema}
      forms={forms}
      views={views}
      viewState={viewState}
      gridDensity={gridDensity}
      activeViewDefaultSort={activeView?.default_sort ?? null}
      isRefreshingRecords={isRefreshingRecords}
      gridColumns={gridColumns}
      deletingRecordId={panelState.deletingRecordId}
      filteredRecords={filteredRecords}
      runtimeRecords={runtimeRecords}
      panelState={panelState}
      refreshErrorMessage={refreshErrorMessage}
      onActiveFormChange={(name) =>
        setViewState((current) => ({ ...current, activeFormLogicalName: name }))
      }
      onActiveViewChange={(name) =>
        setViewState((current) => ({ ...current, activeViewLogicalName: name }))
      }
      onRefresh={() => {
        clearMessages();
        router.refresh();
      }}
      onSearchChange={(recordSearch) =>
        setViewState((current) => ({ ...current, recordSearch }))
      }
      onCreateNew={() => {
        const params = new URLSearchParams();
        if (viewState.activeFormLogicalName) {
          params.set("form", viewState.activeFormLogicalName);
        }
        if (viewState.activeViewLogicalName) {
          params.set("view", viewState.activeViewLogicalName);
        }
        const suffix = params.toString() ? `?${params.toString()}` : "";
        router.push(
          `/worker/apps/${encodeURIComponent(appLogicalName)}/${encodeURIComponent(entityLogicalName)}/new${suffix}`,
        );
      }}
      onViewModeChange={(viewMode) =>
        setViewState((current) => ({ ...current, viewMode }))
      }
      onGridDensityChange={(density) => {
        setGridDensity(density);
        try {
          localStorage.setItem(`worker_grid_density_${appLogicalName}`, density);
        } catch {
          // ignore storage errors
        }
      }}
      onDeleteRecord={handleDeleteRecord}
    />
  );
}

type WorkspaceEntityLayoutProps = {
  appLogicalName: string;
  entityLogicalName: string;
  capabilities: AppEntityCapabilitiesResponse;
  schema: PublishedSchemaResponse;
  forms: ParsedFormResponse[];
  views: ParsedViewResponse[];
  viewState: ViewState;
  gridDensity: WorkerGridDensity;
  activeViewDefaultSort: ParsedViewResponse["default_sort"];
  isRefreshingRecords: boolean;
  gridColumns: Array<ViewColumn & { field: FieldResponse | null }>;
  deletingRecordId: string | null;
  filteredRecords: RuntimeRecordResponse[];
  runtimeRecords: RuntimeRecordResponse[];
  panelState: PanelState;
  refreshErrorMessage: string | null;
  onActiveFormChange: (logicalName: string) => void;
  onActiveViewChange: (logicalName: string) => void;
  onRefresh: () => void;
  onCreateNew: () => void;
  onSearchChange: (value: string) => void;
  onViewModeChange: (mode: WorkerViewMode) => void;
  onGridDensityChange: (density: WorkerGridDensity) => void;
  onDeleteRecord: (recordId: string) => void;
};

function WorkspaceEntityLayout({
  appLogicalName,
  entityLogicalName,
  capabilities,
  schema,
  forms,
  views,
  viewState,
  gridDensity,
  activeViewDefaultSort,
  isRefreshingRecords,
  gridColumns,
  deletingRecordId,
  filteredRecords,
  runtimeRecords,
  panelState,
  refreshErrorMessage,
  onActiveFormChange,
  onActiveViewChange,
  onRefresh,
  onCreateNew,
  onSearchChange,
  onViewModeChange,
  onGridDensityChange,
  onDeleteRecord,
}: WorkspaceEntityLayoutProps) {
  return (
    <div className="space-y-4">
      <section className="space-y-3 rounded-lg border border-emerald-100 bg-white p-3 shadow-sm">
        <WorkspaceToolbar
          capabilities={capabilities}
          filteredRecordCount={filteredRecords.length}
          schemaVersion={schema.version}
          onRefresh={onRefresh}
          onCreateNew={onCreateNew}
          onSearchChange={onSearchChange}
          onViewModeChange={onViewModeChange}
          onDensityChange={onGridDensityChange}
          isRefreshingRecords={isRefreshingRecords}
          recordSearch={viewState.recordSearch}
          viewMode={viewState.viewMode}
          density={gridDensity}
        />
      </section>

      <section className="rounded-lg border border-emerald-100 bg-white p-3 shadow-sm">
        <p className="mb-2 text-[10px] font-semibold uppercase tracking-[0.14em] text-emerald-700">
          View &amp; Form
        </p>
        <div className="grid gap-3 md:grid-cols-2">
          <div className="space-y-1">
            <Label htmlFor="workspace_active_view">Active List View</Label>
            <Select
              id="workspace_active_view"
              value={viewState.activeViewLogicalName ?? ""}
              onChange={(event) => onActiveViewChange(event.target.value)}
            >
              {views.map((view) => (
                <option key={view.logical_name} value={view.logical_name}>
                  {view.display_name}
                </option>
              ))}
            </Select>
          </div>
          <div className="space-y-1">
            <Label htmlFor="workspace_active_form">Active Form</Label>
            <Select
              id="workspace_active_form"
              value={viewState.activeFormLogicalName ?? ""}
              onChange={(event) => onActiveFormChange(event.target.value)}
            >
              {forms.map((form) => (
                <option key={form.logical_name} value={form.logical_name}>
                  {form.display_name} ({form.form_type})
                </option>
              ))}
            </Select>
          </div>
        </div>
      </section>

      <MetadataGrid
        appLogicalName={appLogicalName}
        activeFormLogicalName={viewState.activeFormLogicalName}
        activeViewLogicalName={viewState.activeViewLogicalName}
        capabilities={capabilities}
        columns={gridColumns}
        defaultSort={activeViewDefaultSort}
        deletingRecordId={deletingRecordId}
        entityLogicalName={entityLogicalName}
        filteredRecords={filteredRecords}
        onDeleteRecord={onDeleteRecord}
        optionSets={schema.option_sets}
        records={runtimeRecords}
        viewMode={viewState.viewMode}
        density={gridDensity}
      />

      {panelState.errorMessage ? <Notice tone="error">{panelState.errorMessage}</Notice> : null}
      {panelState.statusMessage ? (
        <Notice tone="success">{panelState.statusMessage}</Notice>
      ) : null}
      {!panelState.errorMessage && refreshErrorMessage ? (
        <Notice tone="warning">{refreshErrorMessage}</Notice>
      ) : null}
    </div>
  );
}

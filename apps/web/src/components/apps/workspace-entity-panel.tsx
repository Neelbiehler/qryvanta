"use client";

import { type FormEvent, useEffect, useMemo, useState } from "react";
import { useRouter } from "next/navigation";

import { Notice } from "@qryvanta/ui";

import {
  apiFetch,
  type AppEntityBindingResponse,
  type AppEntityCapabilitiesResponse,
  type BusinessRuleResponse,
  type CreateRuntimeRecordRequest,
  type FieldResponse,
  type OptionSetResponse,
  type PublishedSchemaResponse,
  type RuntimeRecordResponse,
} from "@/lib/api";
import {
  buildFieldMap,
  buildInitialValues,
  parseFormResponse,
  parseViewResponse,
} from "@/components/apps/workspace-entity/helpers";
import { evaluateRuleState } from "@/components/apps/workspace-entity/business-rules";
import { MetadataDrivenCreateForm } from "@/components/apps/workspace-entity/create-form";
import { MetadataGrid } from "@/components/apps/workspace-entity/metadata-grid";
import {
  WorkspaceToolbar,
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
  businessRules: BusinessRuleResponse[];
  records: RuntimeRecordResponse[];
  forms: ParsedFormResponse[];
  views: ParsedViewResponse[];
  initialFormLogicalName?: string | null;
  initialViewLogicalName?: string | null;
};

type PanelState = {
  errorMessage: string | null;
  statusMessage: string | null;
  isSaving: boolean;
  deletingRecordId: string | null;
};

type ViewState = {
  recordSearch: string;
  viewMode: WorkerViewMode;
  showCreatePanel: boolean;
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
  businessRules,
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

  const [formValues, setFormValues] = useState<Record<string, unknown>>(() =>
    buildInitialValues(schema),
  );
  const [panelState, setPanelState] = useState<PanelState>({
    errorMessage: null,
    statusMessage: null,
    isSaving: false,
    deletingRecordId: null,
  });
  const [viewState, setViewState] = useState<ViewState>({
    recordSearch: "",
    viewMode: binding?.default_view_mode ?? "grid",
    showCreatePanel: capabilities.can_create,
    activeFormLogicalName: defaultFormName,
    activeViewLogicalName: defaultViewName,
  });
  const activeForm = useMemo(
    () => forms.find((f) => f.logical_name === viewState.activeFormLogicalName) ?? forms[0] ?? null,
    [forms, viewState.activeFormLogicalName],
  );

  const activeView = useMemo(
    () => views.find((v) => v.logical_name === viewState.activeViewLogicalName) ?? views[0] ?? null,
    [views, viewState.activeViewLogicalName],
  );

  const evaluatedRuleState = useMemo(
    () => evaluateRuleState(businessRules, activeForm?.logical_name ?? null, formValues),
    [activeForm?.logical_name, businessRules, formValues],
  );

  useEffect(() => {
    if (evaluatedRuleState.valuePatches.size === 0) {
      return;
    }

    setFormValues((current) => {
      let changed = false;
      const next = { ...current };

      for (const [fieldLogicalName, patchedValue] of evaluatedRuleState.valuePatches) {
        if (Object.is(current[fieldLogicalName], patchedValue)) {
          continue;
        }

        next[fieldLogicalName] = patchedValue;
        changed = true;
      }

      return changed ? next : current;
    });
  }, [evaluatedRuleState.valuePatches]);

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

  function setFieldValue(fieldLogicalName: string, value: unknown) {
    setFormValues((current) => ({
      ...current,
      [fieldLogicalName]: value,
    }));
  }

  async function handleCreateRecord(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();

    if (evaluatedRuleState.errorMessages.length > 0) {
      setErrorMessage(evaluatedRuleState.errorMessages.join(" "));
      return;
    }

    if (!capabilities.can_create) {
      setErrorMessage(
        "You do not have create permission for this entity in this app.",
      );
      return;
    }

    clearMessages();
    setPanelState((current) => ({ ...current, isSaving: true }));

    try {
      const payload: CreateRuntimeRecordRequest = {
        data: buildCreatePayloadFromFormState({
          activeForm,
          hiddenFieldNames: evaluatedRuleState.hiddenFieldNames,
          formValues,
          schemaFields: schema.fields,
        }),
      };
      const response = await apiFetch(
        `/api/workspace/apps/${appLogicalName}/entities/${entityLogicalName}/records`,
        {
          method: "POST",
          body: JSON.stringify(payload),
        },
      );

      if (!response.ok) {
        const body = (await response.json()) as { message?: string };
        setErrorMessage(body.message ?? "Unable to create record.");
        return;
      }

      setStatusMessage("Record created.");
      setFormValues(buildInitialValues(schema));
      router.refresh();
    } catch {
      setErrorMessage("Unable to create record.");
    } finally {
      setPanelState((current) => ({ ...current, isSaving: false }));
    }
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
      fieldMap={fieldMap}
      viewState={viewState}
      isRefreshingRecords={isRefreshingRecords}
      activeForm={activeForm}
      formValues={formValues}
      isSaving={panelState.isSaving}
      ruleState={evaluatedRuleState}
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
      onToggleCreatePanel={() =>
        setViewState((current) => ({
          ...current,
          showCreatePanel: !current.showCreatePanel,
        }))
      }
      onViewModeChange={(viewMode) =>
        setViewState((current) => ({ ...current, viewMode }))
      }
      onFieldValueChange={setFieldValue}
      onCreateRecord={handleCreateRecord}
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
  fieldMap: Map<string, FieldResponse>;
  viewState: ViewState;
  isRefreshingRecords: boolean;
  activeForm: ParsedFormResponse | null;
  formValues: Record<string, unknown>;
  isSaving: boolean;
  ruleState: ReturnType<typeof evaluateRuleState>;
  gridColumns: Array<ViewColumn & { field: FieldResponse | null }>;
  deletingRecordId: string | null;
  filteredRecords: RuntimeRecordResponse[];
  runtimeRecords: RuntimeRecordResponse[];
  panelState: PanelState;
  refreshErrorMessage: string | null;
  onActiveFormChange: (logicalName: string) => void;
  onActiveViewChange: (logicalName: string) => void;
  onRefresh: () => void;
  onSearchChange: (value: string) => void;
  onToggleCreatePanel: () => void;
  onViewModeChange: (mode: WorkerViewMode) => void;
  onFieldValueChange: (fieldLogicalName: string, value: unknown) => void;
  onCreateRecord: (event: FormEvent<HTMLFormElement>) => void;
  onDeleteRecord: (recordId: string) => void;
};

function WorkspaceEntityLayout({
  appLogicalName,
  entityLogicalName,
  capabilities,
  schema,
  forms,
  views,
  fieldMap,
  viewState,
  isRefreshingRecords,
  activeForm,
  formValues,
  isSaving,
  ruleState,
  gridColumns,
  deletingRecordId,
  filteredRecords,
  runtimeRecords,
  panelState,
  refreshErrorMessage,
  onActiveFormChange,
  onActiveViewChange,
  onRefresh,
  onSearchChange,
  onToggleCreatePanel,
  onViewModeChange,
  onFieldValueChange,
  onCreateRecord,
  onDeleteRecord,
}: WorkspaceEntityLayoutProps) {
  return (
    <div className="space-y-6">
      <section className="space-y-3 rounded-md border border-zinc-200 bg-zinc-50 p-4">
        <WorkspaceToolbar
          capabilities={capabilities}
          filteredRecordCount={filteredRecords.length}
          schemaVersion={schema.version}
          forms={forms}
          views={views}
          activeFormLogicalName={viewState.activeFormLogicalName}
          activeViewLogicalName={viewState.activeViewLogicalName}
          onActiveFormChange={onActiveFormChange}
          onActiveViewChange={onActiveViewChange}
          onRefresh={onRefresh}
          onSearchChange={onSearchChange}
          onToggleCreatePanel={onToggleCreatePanel}
          onViewModeChange={onViewModeChange}
          isRefreshingRecords={isRefreshingRecords}
          recordSearch={viewState.recordSearch}
          showCreatePanel={viewState.showCreatePanel}
          viewMode={viewState.viewMode}
        />
      </section>

      {viewState.showCreatePanel ? (
        <MetadataDrivenCreateForm
          activeForm={activeForm}
          appLogicalName={appLogicalName}
          canCreate={capabilities.can_create}
          entityDisplayName={schema.entity_display_name}
          fieldMap={fieldMap}
          formValues={formValues}
          isSaving={isSaving}
          optionSets={schema.option_sets}
          onFieldValueChange={onFieldValueChange}
          onSubmit={onCreateRecord}
          ruleState={ruleState}
          schema={schema}
        />
      ) : null}

      <MetadataGrid
        appLogicalName={appLogicalName}
        activeFormLogicalName={viewState.activeFormLogicalName}
        activeViewLogicalName={viewState.activeViewLogicalName}
        capabilities={capabilities}
        columns={gridColumns}
        deletingRecordId={deletingRecordId}
        entityLogicalName={entityLogicalName}
        filteredRecords={filteredRecords}
        onDeleteRecord={onDeleteRecord}
        optionSets={schema.option_sets}
        records={runtimeRecords}
        viewMode={viewState.viewMode}
      />

      {panelState.errorMessage ? <Notice tone="error">{panelState.errorMessage}</Notice> : null}
      {panelState.statusMessage ? (
        <Notice tone="success">{panelState.statusMessage}</Notice>
      ) : null}
      {!panelState.errorMessage && refreshErrorMessage ? (
        <Notice tone="warning">{refreshErrorMessage}</Notice>
      ) : null}
      {ruleState.errorMessages.length > 0 ? (
        <Notice tone="warning">{ruleState.errorMessages.join(" ")}</Notice>
      ) : null}
    </div>
  );
}

function buildCreatePayloadFromFormState(input: {
  activeForm: ParsedFormResponse | null;
  hiddenFieldNames: Set<string>;
  formValues: Record<string, unknown>;
  schemaFields: FieldResponse[];
}): Record<string, unknown> {
  const payload: Record<string, unknown> = {};

  const formFieldNames = new Set<string>();
  if (input.activeForm) {
    for (const tab of input.activeForm.tabs) {
      for (const section of tab.sections) {
        for (const placement of section.fields) {
          formFieldNames.add(placement.field_logical_name);
        }
      }
    }
  }

  const fieldsToProcess =
    formFieldNames.size > 0
      ? input.schemaFields.filter((field) => formFieldNames.has(field.logical_name))
      : input.schemaFields;

  for (const field of fieldsToProcess) {
    const value = input.formValues[field.logical_name];
    if (input.hiddenFieldNames.has(field.logical_name)) {
      continue;
    }

    if (field.field_type === "boolean") {
      payload[field.logical_name] = Boolean(value);
      continue;
    }

    if (typeof value === "string") {
      const trimmed = value.trim();
      if (!trimmed) {
        continue;
      }

      if (field.field_type === "number") {
        payload[field.logical_name] = Number.parseFloat(trimmed);
        continue;
      }

      if (field.field_type === "json") {
        payload[field.logical_name] = JSON.parse(trimmed);
        continue;
      }

      payload[field.logical_name] = trimmed;
      continue;
    }

    if (value !== null && value !== undefined) {
      payload[field.logical_name] = value;
    }
  }

  return payload;
}

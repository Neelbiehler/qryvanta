"use client";

import { type FormEvent, useCallback, useEffect, useMemo, useState } from "react";
import Link from "next/link";
import { useRouter } from "next/navigation";

import {
  Button,
  Checkbox,
  Input,
  Label,
  Notice,
  Select,
  StatusBadge,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
  Textarea,
} from "@qryvanta/ui";

import {
  apiFetch,
  type AppEntityBindingResponse,
  type AppEntityCapabilitiesResponse,
  type BusinessRuleResponse,
  type CreateRuntimeRecordRequest,
  type FieldResponse,
  type OptionSetResponse,
  type PublishedSchemaResponse,
  type QueryRuntimeRecordsRequest,
  type RuntimeRecordResponse,
} from "@/lib/api";
import {
  buildFieldMap,
  buildInitialValues,
  formatFieldValue,
  formatValue,
  parseFormResponse,
  parseViewResponse,
} from "@/components/apps/workspace-entity/helpers";
import { evaluateRuleState } from "@/components/apps/workspace-entity/business-rules";
import { RelationFieldSelect } from "@/components/apps/relation-field-select";
import type {
  FormFieldPlacement,
  FormSection,
  FormTab,
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

type WorkerViewMode = "grid" | "json";

type PanelMessages = {
  errorMessage: string | null;
  statusMessage: string | null;
};

type MutationState = {
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
  const [messages, setMessages] = useState<PanelMessages>({
    errorMessage: null,
    statusMessage: null,
  });
  const [mutationState, setMutationState] = useState<MutationState>({
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
  const [runtimeRecords, setRuntimeRecords] = useState<RuntimeRecordResponse[]>(records);
  const [isRefreshingRecords, setIsRefreshingRecords] = useState(false);

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

  useEffect(() => {
    let cancelled = false;

    async function refreshRecordsForActiveView() {
      if (!activeView) {
        setRuntimeRecords(records);
        return;
      }

      setIsRefreshingRecords(true);

      const payload: QueryRuntimeRecordsRequest = {
        limit: 50,
        offset: 0,
        logical_mode: activeView.filter_criteria?.logical_mode ?? null,
        where: null,
        conditions:
          activeView.filter_criteria?.conditions.map((condition) => ({
            scope_alias: null,
            field_logical_name: condition.field_logical_name,
            operator: condition.operator,
            field_value: condition.value,
          })) ?? null,
        link_entities: null,
        sort: activeView.default_sort
          ? [
              {
                scope_alias: null,
                field_logical_name: activeView.default_sort.field_logical_name,
                direction: activeView.default_sort.direction,
              },
            ]
          : null,
        filters: null,
      };

      try {
        const response = await apiFetch(
          `/api/workspace/apps/${appLogicalName}/entities/${entityLogicalName}/records/query`,
          {
            method: "POST",
            body: JSON.stringify(payload),
          },
        );

        if (!response.ok) {
          const body = (await response.json()) as { message?: string };
          if (!cancelled) {
            setErrorMessage(body.message ?? "Unable to refresh records for selected view.");
          }
          return;
        }

        const nextRecords = (await response.json()) as RuntimeRecordResponse[];
        if (!cancelled) {
          setRuntimeRecords(nextRecords);
        }
      } catch {
        if (!cancelled) {
          setErrorMessage("Unable to refresh records for selected view.");
        }
      } finally {
        if (!cancelled) {
          setIsRefreshingRecords(false);
        }
      }
    }

    void refreshRecordsForActiveView();

    return () => {
      cancelled = true;
    };
  }, [activeView, appLogicalName, entityLogicalName, records]);

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
    setMessages((current) => ({ ...current, errorMessage: next }));
  }

  function setStatusMessage(next: string | null) {
    setMessages((current) => ({ ...current, statusMessage: next }));
  }

  function clearMessages() {
    setMessages({ errorMessage: null, statusMessage: null });
  }

  function setFieldValue(fieldLogicalName: string, value: unknown) {
    setFormValues((current) => ({
      ...current,
      [fieldLogicalName]: value,
    }));
  }

  const buildPayloadFromForm = useCallback(() => {
    const payload: Record<string, unknown> = {};

    // Collect all field logical names from the active form's tabs/sections
    const formFieldNames = new Set<string>();
    if (activeForm) {
      for (const tab of activeForm.tabs) {
        for (const section of tab.sections) {
          for (const fp of section.fields) {
            formFieldNames.add(fp.field_logical_name);
          }
        }
      }
    }

    // Fallback: if no form, use all schema fields
    const fieldsToProcess =
      formFieldNames.size > 0
        ? schema.fields.filter((f) => formFieldNames.has(f.logical_name))
        : schema.fields;

    for (const field of fieldsToProcess) {
      const value = formValues[field.logical_name];
      if (evaluatedRuleState.hiddenFieldNames.has(field.logical_name)) {
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
  }, [activeForm, evaluatedRuleState.hiddenFieldNames, formValues, schema.fields]);

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
    setMutationState((current) => ({ ...current, isSaving: true }));

    try {
      const payload: CreateRuntimeRecordRequest = {
        data: buildPayloadFromForm(),
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
      setMutationState((current) => ({ ...current, isSaving: false }));
    }
  }

  async function handleDeleteRecord(recordId: string) {
    if (!capabilities.can_delete) {
      setErrorMessage(
        "You do not have delete permission for this entity in this app.",
      );
      return;
    }

    setMutationState((current) => ({ ...current, deletingRecordId: recordId }));
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
      setMutationState((current) => ({ ...current, deletingRecordId: null }));
    }
  }

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
          isSaving={mutationState.isSaving}
          optionSets={schema.option_sets}
          onFieldValueChange={setFieldValue}
          onSubmit={handleCreateRecord}
          ruleState={evaluatedRuleState}
          schema={schema}
        />
      ) : null}

      <MetadataGrid
        appLogicalName={appLogicalName}
        activeFormLogicalName={viewState.activeFormLogicalName}
        activeViewLogicalName={viewState.activeViewLogicalName}
        capabilities={capabilities}
        columns={gridColumns}
        deletingRecordId={mutationState.deletingRecordId}
        entityLogicalName={entityLogicalName}
        filteredRecords={filteredRecords}
        onDeleteRecord={handleDeleteRecord}
        optionSets={schema.option_sets}
        records={runtimeRecords}
        viewMode={viewState.viewMode}
      />

      {messages.errorMessage ? (
        <Notice tone="error">{messages.errorMessage}</Notice>
      ) : null}
      {messages.statusMessage ? (
        <Notice tone="success">{messages.statusMessage}</Notice>
      ) : null}
      {evaluatedRuleState.errorMessages.length > 0 ? (
        <Notice tone="warning">{evaluatedRuleState.errorMessages.join(" ")}</Notice>
      ) : null}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Toolbar with form/view selectors
// ---------------------------------------------------------------------------

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

function WorkspaceToolbar({
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
    <>
      <div className="flex flex-wrap items-center justify-between gap-2 border-b border-zinc-200 pb-2">
        <p className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500">
          Command Bar
        </p>
        <p className="text-xs text-zinc-500">Model-driven runtime view</p>
      </div>

      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex flex-wrap items-center gap-2">
          <StatusBadge tone="success">Schema v{schemaVersion}</StatusBadge>
          <StatusBadge tone={capabilities.can_create ? "success" : "warning"}>
            Create {capabilities.can_create ? "Enabled" : "Disabled"}
          </StatusBadge>
          <StatusBadge tone={capabilities.can_delete ? "warning" : "neutral"}>
            Delete {capabilities.can_delete ? "Enabled" : "Disabled"}
          </StatusBadge>
          {isRefreshingRecords ? (
            <StatusBadge tone="neutral">Refreshing records</StatusBadge>
          ) : null}
          <StatusBadge tone="neutral">Records {filteredRecordCount}</StatusBadge>
        </div>

        <div className="flex flex-wrap items-center gap-2">
          <Button
            type="button"
            variant={showCreatePanel ? "default" : "outline"}
            onClick={onToggleCreatePanel}
          >
            {showCreatePanel ? "Hide Quick Create" : "Quick Create"}
          </Button>
          <Button
            type="button"
            variant={viewMode === "grid" ? "default" : "outline"}
            onClick={() => onViewModeChange("grid")}
          >
            Grid View
          </Button>
          <Button
            type="button"
            variant={viewMode === "json" ? "default" : "outline"}
            onClick={() => onViewModeChange("json")}
          >
            JSON View
          </Button>
          <Button type="button" variant="outline" onClick={onRefresh}>
            Refresh
          </Button>
        </div>
      </div>

      {/* Form / View selectors */}
      <div className="flex flex-wrap items-end gap-4">
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
      </div>

      <div className="grid gap-3 md:grid-cols-[1fr_auto]">
        <Input
          value={recordSearch}
          onChange={(event) => onSearchChange(event.target.value)}
          placeholder="Search by record id or field value"
        />
        <p className="text-xs text-zinc-500 md:self-center">
          {filteredRecordCount} visible row(s)
        </p>
      </div>
    </>
  );
}

// ---------------------------------------------------------------------------
// Metadata-driven create form (tab/section/column layout)
// ---------------------------------------------------------------------------

type MetadataDrivenCreateFormProps = {
  activeForm: ParsedFormResponse | null;
  appLogicalName: string;
  canCreate: boolean;
  entityDisplayName: string;
  fieldMap: Map<string, FieldResponse>;
  formValues: Record<string, unknown>;
  isSaving: boolean;
  optionSets: OptionSetResponse[];
  onFieldValueChange: (fieldLogicalName: string, value: unknown) => void;
  onSubmit: (event: FormEvent<HTMLFormElement>) => void;
  ruleState: ReturnType<typeof evaluateRuleState>;
  schema: PublishedSchemaResponse;
};

function MetadataDrivenCreateForm({
  activeForm,
  appLogicalName,
  canCreate,
  entityDisplayName,
  fieldMap,
  formValues,
  isSaving,
  optionSets,
  onFieldValueChange,
  onSubmit,
  ruleState,
  schema,
}: MetadataDrivenCreateFormProps) {
  // If no form definition is available, fall back to a flat field list
  if (!activeForm || activeForm.tabs.length === 0) {
    return (
      <FlatCreateForm
        appLogicalName={appLogicalName}
        canCreate={canCreate}
        entityDisplayName={entityDisplayName}
        fields={schema.fields}
        formValues={formValues}
        isSaving={isSaving}
        optionSets={optionSets}
        onFieldValueChange={onFieldValueChange}
        onSubmit={onSubmit}
        ruleState={ruleState}
      />
    );
  }

  const visibleTabs = activeForm.tabs.filter((tab) => tab.visible);
  const showTabHeaders = visibleTabs.length > 1;

  return (
    <form
      className="space-y-4 rounded-md border border-emerald-100 bg-white p-4"
      onSubmit={onSubmit}
    >
      <div>
        <p className="text-sm font-medium text-zinc-800">
          New {entityDisplayName} Record
        </p>
        <p className="text-xs text-zinc-500">
          Using form: {activeForm.display_name} ({activeForm.form_type}) in {appLogicalName}.
        </p>
      </div>

      {visibleTabs.map((tab) => (
        <FormTabRenderer
          key={tab.logical_name}
          appLogicalName={appLogicalName}
          tab={tab}
          fieldMap={fieldMap}
          formValues={formValues}
          onFieldValueChange={onFieldValueChange}
          optionSets={optionSets}
          ruleState={ruleState}
          showTabHeader={showTabHeaders}
        />
      ))}

      <Button disabled={!canCreate || isSaving} type="submit">
        {isSaving ? "Saving..." : "Create Record"}
      </Button>
    </form>
  );
}

// ---------------------------------------------------------------------------
// Tab renderer
// ---------------------------------------------------------------------------

type FormTabRendererProps = {
  appLogicalName: string;
  tab: FormTab;
  fieldMap: Map<string, FieldResponse>;
  formValues: Record<string, unknown>;
  onFieldValueChange: (fieldLogicalName: string, value: unknown) => void;
  optionSets: OptionSetResponse[];
  ruleState: ReturnType<typeof evaluateRuleState>;
  showTabHeader: boolean;
};

function FormTabRenderer({
  appLogicalName,
  tab,
  fieldMap,
  formValues,
  onFieldValueChange,
  optionSets,
  ruleState,
  showTabHeader,
}: FormTabRendererProps) {
  const visibleSections = tab.sections
    .filter((section) => section.visible)
    .sort((a, b) => a.position - b.position);

  return (
    <div className="space-y-4">
      {showTabHeader ? (
        <div className="border-b border-zinc-200 pb-1">
          <p className="text-sm font-semibold text-zinc-700">{tab.display_name}</p>
        </div>
      ) : null}

      {visibleSections.map((section) => (
        <FormSectionRenderer
          key={section.logical_name}
          appLogicalName={appLogicalName}
          section={section}
          fieldMap={fieldMap}
          formValues={formValues}
          onFieldValueChange={onFieldValueChange}
          optionSets={optionSets}
          ruleState={ruleState}
        />
      ))}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Section renderer
// ---------------------------------------------------------------------------

type FormSectionRendererProps = {
  appLogicalName: string;
  section: FormSection;
  fieldMap: Map<string, FieldResponse>;
  formValues: Record<string, unknown>;
  onFieldValueChange: (fieldLogicalName: string, value: unknown) => void;
  optionSets: OptionSetResponse[];
  ruleState: ReturnType<typeof evaluateRuleState>;
};

function FormSectionRenderer({
  appLogicalName,
  section,
  fieldMap,
  formValues,
  onFieldValueChange,
  optionSets,
  ruleState,
}: FormSectionRendererProps) {
  const visibleFields = section.fields
    .filter((fp) => fp.visible && !ruleState.hiddenFieldNames.has(fp.field_logical_name))
    .sort((a, b) => a.position - b.position);

  if (visibleFields.length === 0) {
    return null;
  }

  // Group fields by column
  const columnGroups: FormFieldPlacement[][] = [];
  for (let i = 0; i < section.columns; i++) {
    columnGroups.push(
      visibleFields
        .filter((fp) => fp.column === i)
        .sort((a, b) => a.position - b.position),
    );
  }

  const gridClass =
    section.columns === 3
      ? "grid gap-4 md:grid-cols-3"
      : section.columns === 2
        ? "grid gap-4 md:grid-cols-2"
        : "grid gap-4 grid-cols-1";

  return (
    <fieldset className="space-y-3">
      <legend className="text-xs font-semibold uppercase tracking-[0.12em] text-zinc-500">
        {section.display_name}
      </legend>
      <div className={gridClass}>
        {columnGroups.map((columnFields, colIndex) => (
          <div key={`col-${String(colIndex)}`} className="space-y-4">
            {columnFields.map((fp) => (
              <FieldControl
                key={fp.field_logical_name}
                appLogicalName={appLogicalName}
                placement={fp}
                field={fieldMap.get(fp.field_logical_name) ?? null}
                formValues={formValues}
                onFieldValueChange={onFieldValueChange}
                optionSets={optionSets}
                ruleState={ruleState}
              />
            ))}
          </div>
        ))}
      </div>

      {section.subgrids.length > 0 ? (
        <Notice tone="warning">
          This section includes {section.subgrids.length} sub-grid control(s). Sub-grids render on saved record detail pages.
        </Notice>
      ) : null}
    </fieldset>
  );
}

// ---------------------------------------------------------------------------
// Individual field control (option-set aware)
// ---------------------------------------------------------------------------

type FieldControlProps = {
  appLogicalName: string;
  placement: FormFieldPlacement;
  field: FieldResponse | null;
  formValues: Record<string, unknown>;
  onFieldValueChange: (fieldLogicalName: string, value: unknown) => void;
  optionSets: OptionSetResponse[];
  ruleState: ReturnType<typeof evaluateRuleState>;
};

function FieldControl({
  appLogicalName,
  placement,
  field,
  formValues,
  onFieldValueChange,
  optionSets,
  ruleState,
}: FieldControlProps) {
  if (!field) {
    return (
      <div className="text-xs text-zinc-400">
        Unknown field: {placement.field_logical_name}
      </div>
    );
  }

  const fieldId = `field_${field.logical_name}`;
  const value = formValues[field.logical_name];
  const displayLabel = placement.label_override ?? field.display_name;
  const isRequired =
    ruleState.requiredOverrides.get(field.logical_name) ??
    (placement.required_override !== null
      ? placement.required_override
      : field.is_required);
  const isReadOnly =
    ruleState.readOnlyOverrides.get(field.logical_name) ?? placement.read_only;

  // Option set field
  if (field.option_set_logical_name) {
    const optionSet = optionSets.find(
      (os) => os.logical_name === field.option_set_logical_name,
    );
    if (optionSet) {
      return (
        <div className="space-y-2">
          <Label htmlFor={fieldId}>
            {displayLabel}
            {isRequired ? <span className="text-red-500"> *</span> : null}
          </Label>
          <Select
            id={fieldId}
            value={String(value ?? "")}
            onChange={(event) => {
              const numValue = Number(event.target.value);
              onFieldValueChange(
                field.logical_name,
                Number.isNaN(numValue) ? event.target.value : numValue,
              );
            }}
            disabled={isReadOnly}
            required={isRequired}
          >
            <option value="">-- Select --</option>
            {[...optionSet.options]
              .sort((a, b) => a.position - b.position)
              .map((opt) => (
                <option key={opt.value} value={opt.value}>
                  {opt.label}
                </option>
              ))}
          </Select>
        </div>
      );
    }
  }

  if (field.field_type === "relation" && field.relation_target_entity) {
    return (
      <div className="space-y-2">
        <Label htmlFor={fieldId}>
          {displayLabel}
          {isRequired ? <span className="text-red-500"> *</span> : null}
        </Label>
        <RelationFieldSelect
          appLogicalName={appLogicalName}
          entityLogicalName={field.relation_target_entity}
          fieldId={fieldId}
          value={value}
          disabled={isReadOnly}
          required={isRequired}
          onChange={(nextValue) => onFieldValueChange(field.logical_name, nextValue)}
        />
      </div>
    );
  }

  // Boolean field
  if (field.field_type === "boolean") {
    return (
      <div className="space-y-2">
        <Label htmlFor={fieldId}>{displayLabel}</Label>
        <label className="inline-flex items-center gap-2 text-sm text-zinc-700">
          <Checkbox
            id={fieldId}
            checked={Boolean(value)}
            onChange={(event) =>
              onFieldValueChange(field.logical_name, event.target.checked)
            }
            disabled={isReadOnly}
          />
          {displayLabel}
        </label>
      </div>
    );
  }

  // JSON field
  if (field.field_type === "json") {
    return (
      <div className="space-y-2">
        <Label htmlFor={fieldId}>
          {displayLabel}
          {isRequired ? <span className="text-red-500"> *</span> : null}
        </Label>
        <Textarea
          id={fieldId}
          className="font-mono text-xs"
          value={String(value ?? "")}
          onChange={(event) =>
            onFieldValueChange(field.logical_name, event.target.value)
          }
          placeholder='{"value":"example"}'
          readOnly={isReadOnly}
          required={isRequired}
        />
      </div>
    );
  }

  // Standard field
  return (
    <div className="space-y-2">
      <Label htmlFor={fieldId}>
        {displayLabel}
        {isRequired ? <span className="text-red-500"> *</span> : null}
      </Label>
      <Input
        id={fieldId}
        type={
          field.field_type === "number"
            ? "number"
            : field.field_type === "date"
              ? "date"
              : field.field_type === "datetime"
                ? "datetime-local"
                : "text"
        }
        value={String(value ?? "")}
        onChange={(event) =>
          onFieldValueChange(field.logical_name, event.target.value)
        }
        required={isRequired}
        readOnly={isReadOnly}
      />
    </div>
  );
}

// ---------------------------------------------------------------------------
// Flat create form fallback (no FormDefinition available)
// ---------------------------------------------------------------------------

type FlatCreateFormProps = {
  appLogicalName: string;
  canCreate: boolean;
  entityDisplayName: string;
  fields: FieldResponse[];
  formValues: Record<string, unknown>;
  isSaving: boolean;
  optionSets: OptionSetResponse[];
  onFieldValueChange: (fieldLogicalName: string, value: unknown) => void;
  onSubmit: (event: FormEvent<HTMLFormElement>) => void;
  ruleState: ReturnType<typeof evaluateRuleState>;
};

function FlatCreateForm({
  appLogicalName,
  canCreate,
  entityDisplayName,
  fields,
  formValues,
  isSaving,
  optionSets,
  onFieldValueChange,
  onSubmit,
  ruleState,
}: FlatCreateFormProps) {
  return (
    <form
      className="space-y-4 rounded-md border border-emerald-100 bg-white p-4"
      onSubmit={onSubmit}
    >
      <div>
        <p className="text-sm font-medium text-zinc-800">New {entityDisplayName} Record</p>
        <p className="text-xs text-zinc-500">
          Fill fields and create a runtime row in {appLogicalName}.
        </p>
      </div>

      <div className="grid gap-4 md:grid-cols-2">
        {fields
          .filter((field) => !ruleState.hiddenFieldNames.has(field.logical_name))
          .map((field) => (
          <FieldControl
            key={field.logical_name}
            appLogicalName={appLogicalName}
            placement={{
              field_logical_name: field.logical_name,
              column: 0,
              position: 0,
              visible: true,
              read_only: false,
              required_override: null,
              label_override: null,
            }}
            field={field}
            formValues={formValues}
            onFieldValueChange={onFieldValueChange}
            optionSets={optionSets}
            ruleState={ruleState}
          />
          ))}
      </div>

      <Button disabled={!canCreate || isSaving} type="submit">
        {isSaving ? "Saving..." : "Create Record"}
      </Button>
    </form>
  );
}

// ---------------------------------------------------------------------------
// Metadata-driven grid (uses ViewDefinition columns, sort, label overrides)
// ---------------------------------------------------------------------------

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
};

function MetadataGrid({
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
}: MetadataGridProps) {
  const queryParams = new URLSearchParams();
  if (activeFormLogicalName) {
    queryParams.set("form", activeFormLogicalName);
  }
  if (activeViewLogicalName) {
    queryParams.set("view", activeViewLogicalName);
  }
  const detailSuffix = queryParams.toString().length > 0 ? `?${queryParams.toString()}` : "";

  return (
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead>Record ID</TableHead>
          {viewMode === "grid"
            ? columns.map((col) => (
                <TableHead
                  key={col.field_logical_name}
                  style={col.width ? { width: `${String(col.width)}px` } : undefined}
                >
                  {col.label_override ?? col.field?.display_name ?? col.field_logical_name}
                </TableHead>
              ))
            : null}
          <TableHead>{viewMode === "grid" ? "Snapshot" : "Data"}</TableHead>
          <TableHead>Actions</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {filteredRecords.length > 0 ? (
          filteredRecords.map((record) => (
            <TableRow key={record.record_id}>
              <TableCell className="font-mono text-xs">
                <Link
                  href={`/worker/apps/${appLogicalName}/${entityLogicalName}/${record.record_id}${detailSuffix}`}
                  className="text-emerald-600 underline-offset-2 hover:underline"
                >
                  {record.record_id}
                </Link>
              </TableCell>
              {viewMode === "grid"
                ? columns.map((col) => (
                    <TableCell
                      className="max-w-[220px] truncate"
                      key={`${record.record_id}-${col.field_logical_name}`}
                      style={col.width ? { width: `${String(col.width)}px` } : undefined}
                      title={
                        col.field
                          ? formatFieldValue(record.data[col.field_logical_name], col.field, optionSets)
                          : formatValue(record.data[col.field_logical_name])
                      }
                    >
                      {col.field
                        ? formatFieldValue(record.data[col.field_logical_name], col.field, optionSets)
                        : formatValue(record.data[col.field_logical_name])}
                    </TableCell>
                  ))
                : null}
              <TableCell className="font-mono text-xs">
                {viewMode === "grid"
                  ? `${String(Object.keys(record.data).length)} populated field(s)`
                  : JSON.stringify(record.data)}
              </TableCell>
              <TableCell>
                {capabilities.can_delete ? (
                  <Button
                    disabled={deletingRecordId === record.record_id}
                    variant="outline"
                    size="sm"
                    type="button"
                    onClick={() => onDeleteRecord(record.record_id)}
                  >
                    {deletingRecordId === record.record_id ? "Deleting..." : "Delete"}
                  </Button>
                ) : (
                  <span className="text-xs text-zinc-500">No delete access</span>
                )}
              </TableCell>
            </TableRow>
          ))
        ) : (
          <TableRow>
            <TableCell
              className="text-zinc-500"
              colSpan={viewMode === "grid" ? columns.length + 3 : 3}
            >
              {records.length > 0 ? "No records match this search." : "No records yet."}
            </TableCell>
          </TableRow>
        )}
      </TableBody>
    </Table>
  );
}

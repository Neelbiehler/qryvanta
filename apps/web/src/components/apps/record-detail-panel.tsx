"use client";

import {
  type FormEvent,
  useCallback,
  useEffect,
  useMemo,
  useReducer,
  useState,
} from "react";
import { useRouter } from "next/navigation";

import {
  Button,
  CommandBar,
  CommandBarAction,
  Label,
  Notice,
  Select,
} from "@qryvanta/ui";
import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "@qryvanta/ui/accordion";

import {
  apiFetch,
  type AppEntityCapabilitiesResponse,
  type BusinessRuleResponse,
  type FieldResponse,
  type OptionSetResponse,
  type PublishedSchemaResponse,
  type RuntimeRecordResponse,
  type UpdateRuntimeRecordRequest,
} from "@/lib/api";
import { buildFieldMap, formatFieldValue } from "@/components/apps/workspace-entity/helpers";
import { evaluateRuleState } from "@/components/apps/workspace-entity/business-rules";
import { RelatedRecordsSubgrid } from "@/components/apps/related-records-subgrid";
import { FieldControl } from "@/components/shared/field-control";
import type {
  FormFieldPlacement,
  FormSection,
  FormSubgrid,
  FormTab,
  ParsedFormResponse,
} from "@/components/apps/workspace-entity/metadata-types";

// ---------------------------------------------------------------------------
// Props
// ---------------------------------------------------------------------------

type RecordDetailPanelProps = {
  appLogicalName: string;
  entityLogicalName: string;
  capabilities: AppEntityCapabilitiesResponse;
  forms: ParsedFormResponse[];
  businessRules: BusinessRuleResponse[];
  initialFormLogicalName?: string | null;
  record: RuntimeRecordResponse;
  schema: PublishedSchemaResponse;
};

type PanelState = {
  errorMessage: string | null;
  statusMessage: string | null;
  isSaving: boolean;
};

type PanelAction =
  | { type: "start_saving" }
  | { type: "set_error"; message: string }
  | { type: "set_success"; message: string }
  | { type: "clear_messages" }
  | { type: "finish_saving" };

function panelStateReducer(state: PanelState, action: PanelAction): PanelState {
  switch (action.type) {
    case "start_saving":
      return { ...state, isSaving: true, errorMessage: null, statusMessage: null };
    case "set_error":
      return { ...state, errorMessage: action.message, statusMessage: null };
    case "set_success":
      return { ...state, statusMessage: action.message, errorMessage: null };
    case "clear_messages":
      return { ...state, errorMessage: null, statusMessage: null };
    case "finish_saving":
      return { ...state, isSaving: false };
    default:
      return state;
  }
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export function RecordDetailPanel({
  appLogicalName,
  entityLogicalName,
  capabilities,
  forms,
  businessRules,
  initialFormLogicalName,
  record,
  schema,
}: RecordDetailPanelProps) {
  const router = useRouter();
  const fieldMap = useMemo(() => buildFieldMap(schema), [schema]);

  // Pick the main form (or first available)
  const mainForms = forms.filter((f) => f.form_type === "main");
  const [activeFormName, setActiveFormName] = useState<string | null>(
    initialFormLogicalName ?? mainForms[0]?.logical_name ?? forms[0]?.logical_name ?? null,
  );
  const activeForm = useMemo(
    () => forms.find((f) => f.logical_name === activeFormName) ?? forms[0] ?? null,
    [forms, activeFormName],
  );

  // Initialise edit values from the record data
  const [formValues, setFormValues] = useState<Record<string, unknown>>(() => {
    const values: Record<string, unknown> = {};
    for (const field of schema.fields) {
      const existing = record.data[field.logical_name];
      if (existing !== undefined && existing !== null) {
        values[field.logical_name] = existing;
      } else if (field.field_type === "boolean") {
        values[field.logical_name] = false;
      } else {
        values[field.logical_name] = "";
      }
    }
    return values;
  });

  const [panelState, dispatchPanel] = useReducer(panelStateReducer, {
    errorMessage: null,
    statusMessage: null,
    isSaving: false,
  });

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

  function setFieldValue(fieldLogicalName: string, value: unknown) {
    setFormValues((current) => ({
      ...current,
      [fieldLogicalName]: value,
    }));
  }

  const buildPayloadFromForm = useCallback(() => {
    const payload: Record<string, unknown> = {};

    // Collect fields from active form if available
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

  async function handleUpdateRecord(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();

    if (evaluatedRuleState.errorMessages.length > 0) {
      dispatchPanel({
        type: "set_error",
        message: evaluatedRuleState.errorMessages.join(" "),
      });
      return;
    }

    if (!capabilities.can_update) {
      dispatchPanel({
        type: "set_error",
        message: "You do not have update permission for this entity.",
      });
      return;
    }

    dispatchPanel({ type: "start_saving" });

    try {
      const payload: UpdateRuntimeRecordRequest = {
        data: buildPayloadFromForm(),
      };
      const response = await apiFetch(
        `/api/workspace/apps/${appLogicalName}/entities/${entityLogicalName}/records/${record.record_id}`,
        {
          method: "PUT",
          body: JSON.stringify(payload),
        },
      );

      if (!response.ok) {
        const body = (await response.json()) as { message?: string };
        dispatchPanel({
          type: "set_error",
          message: body.message ?? "Unable to update record.",
        });
        return;
      }

      dispatchPanel({
        type: "set_success",
        message: "Record updated successfully.",
      });
      router.refresh();
    } catch {
      dispatchPanel({ type: "set_error", message: "Unable to update record." });
    } finally {
      dispatchPanel({ type: "finish_saving" });
    }
  }

  // If no form definition, render flat field layout
  if (!activeForm || activeForm.tabs.length === 0) {
    return (
      <FlatRecordDetail
        appLogicalName={appLogicalName}
        canUpdate={capabilities.can_update}
        fields={schema.fields}
        formValues={formValues}
        isSaving={panelState.isSaving}
        optionSets={schema.option_sets}
        ruleState={evaluatedRuleState}
        onFieldValueChange={setFieldValue}
        onSubmit={handleUpdateRecord}
        errorMessage={panelState.errorMessage}
        statusMessage={panelState.statusMessage}
      />
    );
  }

  const visibleTabs = activeForm.tabs.filter((tab) => tab.visible);
  const showTabHeaders = visibleTabs.length > 1;

  return (
    <RecordDetailWorkspace
      appLogicalName={appLogicalName}
      capabilities={capabilities}
      forms={forms}
      activeForm={activeForm}
      activeFormName={activeFormName}
      onActiveFormNameChange={setActiveFormName}
      fieldMap={fieldMap}
      record={record}
      optionSets={schema.option_sets}
      visibleTabs={visibleTabs}
      formValues={formValues}
      onFieldValueChange={setFieldValue}
      ruleState={evaluatedRuleState}
      showTabHeaders={showTabHeaders}
      onSubmit={handleUpdateRecord}
      isSaving={panelState.isSaving}
      errorMessage={panelState.errorMessage}
      statusMessage={panelState.statusMessage}
    />
  );
}

type RecordDetailWorkspaceProps = {
  appLogicalName: string;
  capabilities: AppEntityCapabilitiesResponse;
  forms: ParsedFormResponse[];
  activeForm: ParsedFormResponse;
  activeFormName: string | null;
  onActiveFormNameChange: (value: string | null) => void;
  fieldMap: Map<string, FieldResponse>;
  record: RuntimeRecordResponse;
  optionSets: OptionSetResponse[];
  visibleTabs: FormTab[];
  formValues: Record<string, unknown>;
  onFieldValueChange: (fieldLogicalName: string, value: unknown) => void;
  ruleState: ReturnType<typeof evaluateRuleState>;
  showTabHeaders: boolean;
  onSubmit: (event: FormEvent<HTMLFormElement>) => void;
  isSaving: boolean;
  errorMessage: string | null;
  statusMessage: string | null;
};

function RecordDetailWorkspace({
  appLogicalName,
  capabilities,
  forms,
  activeForm,
  activeFormName,
  onActiveFormNameChange,
  fieldMap,
  record,
  optionSets,
  visibleTabs,
  formValues,
  onFieldValueChange,
  ruleState,
  showTabHeaders,
  onSubmit,
  isSaving,
  errorMessage,
  statusMessage,
}: RecordDetailWorkspaceProps) {
  return (
    <div className="space-y-6">
      {forms.length > 1 ? (
        <div className="flex items-end gap-4">
          <div className="space-y-1">
            <Label htmlFor="record-form-selector">Form</Label>
            <Select
              id="record-form-selector"
              value={activeFormName ?? ""}
              onChange={(event) => onActiveFormNameChange(event.target.value)}
            >
              {forms.map((form) => (
                <option key={form.logical_name} value={form.logical_name}>
                  {form.display_name} ({form.form_type})
                </option>
              ))}
            </Select>
          </div>
        </div>
      ) : null}

      {activeForm.header_fields.length > 0 ? (
        <div className="flex flex-wrap gap-4 border-b border-zinc-200 pb-3">
          {activeForm.header_fields.map((headerFieldName) => {
            const field = fieldMap.get(headerFieldName);
            if (!field) return null;
            return (
              <div key={headerFieldName} className="space-y-0.5">
                <p className="text-[11px] font-semibold uppercase tracking-[0.12em] text-zinc-500">
                  {field.display_name}
                </p>
                <p className="text-sm font-medium text-zinc-900">
                  {formatFieldValue(record.data[headerFieldName], field, optionSets)}
                </p>
              </div>
            );
          })}
        </div>
      ) : null}

      <form className="space-y-6" onSubmit={onSubmit}>
        {visibleTabs.map((tab) => (
          <RecordTabRenderer
            key={tab.logical_name}
            appLogicalName={appLogicalName}
            currentRecordId={record.record_id}
            tab={tab}
            canUpdate={capabilities.can_update}
            fieldMap={fieldMap}
            formValues={formValues}
            onFieldValueChange={onFieldValueChange}
            optionSets={optionSets}
            ruleState={ruleState}
            showTabHeader={showTabHeaders}
          />
        ))}

        <CommandBar className="rounded-md border border-zinc-200 bg-zinc-50 px-3 py-2">
          {capabilities.can_update ? (
            <CommandBarAction disabled={isSaving} type="submit" variant="primary">
              {isSaving ? "Saving..." : "Save Changes"}
            </CommandBarAction>
          ) : (
            <p className="text-sm text-zinc-500">
              Read-only: you do not have update permission for this entity.
            </p>
          )}
        </CommandBar>
      </form>

      {errorMessage ? <Notice tone="error">{errorMessage}</Notice> : null}
      {statusMessage ? <Notice tone="success">{statusMessage}</Notice> : null}
      {ruleState.errorMessages.length > 0 ? (
        <Notice tone="warning">{ruleState.errorMessages.join(" ")}</Notice>
      ) : null}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Tab renderer for record detail
// ---------------------------------------------------------------------------

type RecordTabRendererProps = {
  appLogicalName: string;
  currentRecordId: string;
  tab: FormTab;
  canUpdate: boolean;
  fieldMap: Map<string, FieldResponse>;
  formValues: Record<string, unknown>;
  onFieldValueChange: (fieldLogicalName: string, value: unknown) => void;
  optionSets: OptionSetResponse[];
  ruleState: ReturnType<typeof evaluateRuleState>;
  showTabHeader: boolean;
};

function RecordTabRenderer({
  appLogicalName,
  currentRecordId,
  tab,
  canUpdate,
  fieldMap,
  formValues,
  onFieldValueChange,
  optionSets,
  ruleState,
  showTabHeader,
}: RecordTabRendererProps) {
  const visibleSections = tab.sections
    .filter((section) => section.visible)
    .sort((a, b) => a.position - b.position);
  const [openSectionValues, setOpenSectionValues] = useState<string[]>(() =>
    visibleSections.length > 0 ? [visibleSections[0].logical_name] : [],
  );

  return (
    <div className="space-y-4">
      {showTabHeader ? (
        <div className="border-b border-zinc-200 pb-1">
          <p className="text-sm font-semibold text-zinc-700">{tab.display_name}</p>
        </div>
      ) : null}

      <Accordion
        type="multiple"
        value={openSectionValues}
        onValueChange={setOpenSectionValues}
        className="space-y-2"
      >
        {visibleSections.map((section) => (
          <AccordionItem
            key={section.logical_name}
            value={section.logical_name}
            className="rounded-md border border-zinc-200 bg-white px-3"
          >
            <AccordionTrigger className="py-2 text-xs uppercase tracking-[0.12em] text-zinc-600">
              {section.display_name}
            </AccordionTrigger>
            <AccordionContent className="pt-2">
              <RecordSectionRenderer
                appLogicalName={appLogicalName}
                currentRecordId={currentRecordId}
                section={section}
                canUpdate={canUpdate}
                fieldMap={fieldMap}
                formValues={formValues}
                onFieldValueChange={onFieldValueChange}
                optionSets={optionSets}
                ruleState={ruleState}
                showSectionTitle={false}
              />
            </AccordionContent>
          </AccordionItem>
        ))}
      </Accordion>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Section renderer for record detail
// ---------------------------------------------------------------------------

type RecordSectionRendererProps = {
  appLogicalName: string;
  currentRecordId: string;
  section: FormSection;
  canUpdate: boolean;
  fieldMap: Map<string, FieldResponse>;
  formValues: Record<string, unknown>;
  onFieldValueChange: (fieldLogicalName: string, value: unknown) => void;
  optionSets: OptionSetResponse[];
  ruleState: ReturnType<typeof evaluateRuleState>;
  showSectionTitle?: boolean;
};

function RecordSectionRenderer({
  appLogicalName,
  currentRecordId,
  section,
  canUpdate,
  fieldMap,
  formValues,
  onFieldValueChange,
  optionSets,
  ruleState,
  showSectionTitle = true,
}: RecordSectionRendererProps) {
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
      {showSectionTitle ? (
        <legend className="text-xs font-semibold uppercase tracking-[0.12em] text-zinc-500">
          {section.display_name}
        </legend>
      ) : null}
      <div className={gridClass}>
        {columnGroups.map((columnFields, colIndex) => (
          <div key={`col-${String(colIndex)}`} className="space-y-4">
            {columnFields.map((fp) => (
              <FieldControl
                key={fp.field_logical_name}
                appLogicalName={appLogicalName}
                placement={fp}
                canEdit={canUpdate}
                field={fieldMap.get(fp.field_logical_name) ?? null}
                formValues={formValues}
                onFieldValueChange={onFieldValueChange}
                optionSets={optionSets}
                ruleState={ruleState}
                fieldIdPrefix="record_field"
                prettyJsonObjects
                jsonRows={4}
              />
            ))}
          </div>
        ))}
      </div>

      {section.subgrids.length > 0 ? (
        <div className="space-y-3">
          {section.subgrids
            .sort((left, right) => left.position - right.position)
            .map((subgrid) => (
              <RecordSubgrid
                key={subgrid.logical_name}
                appLogicalName={appLogicalName}
                currentRecordId={currentRecordId}
                subgrid={subgrid}
              />
            ))}
        </div>
      ) : null}
    </fieldset>
  );
}

type RecordSubgridProps = {
  appLogicalName: string;
  currentRecordId: string;
  subgrid: FormSubgrid;
};

function RecordSubgrid({ appLogicalName, currentRecordId, subgrid }: RecordSubgridProps) {
  if (
    !subgrid.target_entity_logical_name ||
    !subgrid.relation_field_logical_name
  ) {
    return (
      <Notice tone="warning">
        Sub-grid &quot;{subgrid.display_name}&quot; is missing target entity or relation field configuration.
      </Notice>
    );
  }

  return (
    <RelatedRecordsSubgrid
      appLogicalName={appLogicalName}
      currentRecordId={currentRecordId}
      displayName={subgrid.display_name}
      targetEntityLogicalName={subgrid.target_entity_logical_name}
      relationFieldLogicalName={subgrid.relation_field_logical_name}
      columns={subgrid.columns}
    />
  );
}

// ---------------------------------------------------------------------------
// Flat record detail (fallback when no FormDefinition)
// ---------------------------------------------------------------------------

type FlatRecordDetailProps = {
  appLogicalName: string;
  canUpdate: boolean;
  fields: FieldResponse[];
  formValues: Record<string, unknown>;
  isSaving: boolean;
  optionSets: OptionSetResponse[];
  onFieldValueChange: (fieldLogicalName: string, value: unknown) => void;
  onSubmit: (event: FormEvent<HTMLFormElement>) => void;
  ruleState: ReturnType<typeof evaluateRuleState>;
  errorMessage: string | null;
  statusMessage: string | null;
};

function FlatRecordDetail({
  appLogicalName,
  canUpdate,
  fields,
  formValues,
  isSaving,
  optionSets,
  onFieldValueChange,
  onSubmit,
  ruleState,
  errorMessage,
  statusMessage,
}: FlatRecordDetailProps) {
  return (
    <div className="space-y-6">
      <form className="space-y-4" onSubmit={onSubmit}>
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
              canEdit={canUpdate}
              field={field}
              formValues={formValues}
              onFieldValueChange={onFieldValueChange}
              optionSets={optionSets}
              ruleState={ruleState}
              fieldIdPrefix="record_field"
              prettyJsonObjects
              jsonRows={4}
            />
            ))}
        </div>

        {canUpdate ? (
          <Button disabled={isSaving} type="submit">
            {isSaving ? "Saving..." : "Save Changes"}
          </Button>
        ) : (
          <p className="text-sm text-zinc-500">
            Read-only: you do not have update permission for this entity.
          </p>
        )}
      </form>

      {errorMessage ? <Notice tone="error">{errorMessage}</Notice> : null}
      {statusMessage ? <Notice tone="success">{statusMessage}</Notice> : null}
    </div>
  );
}

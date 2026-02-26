"use client";

import { type FormEvent, useCallback, useEffect, useMemo, useState } from "react";
import { useRouter } from "next/navigation";

import {
  Button,
  Checkbox,
  Input,
  Label,
  Notice,
  Select,
  Textarea,
} from "@qryvanta/ui";

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
import { RelationFieldSelect } from "@/components/apps/relation-field-select";
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

  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);
  const [isSaving, setIsSaving] = useState(false);

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
      setErrorMessage(evaluatedRuleState.errorMessages.join(" "));
      return;
    }

    if (!capabilities.can_update) {
      setErrorMessage("You do not have update permission for this entity.");
      return;
    }

    setErrorMessage(null);
    setStatusMessage(null);
    setIsSaving(true);

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
        setErrorMessage(body.message ?? "Unable to update record.");
        return;
      }

      setStatusMessage("Record updated successfully.");
      router.refresh();
    } catch {
      setErrorMessage("Unable to update record.");
    } finally {
      setIsSaving(false);
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
        isSaving={isSaving}
        optionSets={schema.option_sets}
        ruleState={evaluatedRuleState}
        onFieldValueChange={setFieldValue}
        onSubmit={handleUpdateRecord}
        errorMessage={errorMessage}
        statusMessage={statusMessage}
      />
    );
  }

  const visibleTabs = activeForm.tabs.filter((tab) => tab.visible);
  const showTabHeaders = visibleTabs.length > 1;

  return (
    <div className="space-y-6">
      {/* Form selector when multiple forms exist */}
      {forms.length > 1 ? (
        <div className="flex items-end gap-4">
          <div className="space-y-1">
            <Label htmlFor="record-form-selector">Form</Label>
            <Select
              id="record-form-selector"
              value={activeFormName ?? ""}
              onChange={(event) => setActiveFormName(event.target.value)}
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

      {/* Header fields */}
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
                  {formatFieldValue(record.data[headerFieldName], field, schema.option_sets)}
                </p>
              </div>
            );
          })}
        </div>
      ) : null}

      <form className="space-y-6" onSubmit={handleUpdateRecord}>
        {visibleTabs.map((tab) => (
          <RecordTabRenderer
            key={tab.logical_name}
            appLogicalName={appLogicalName}
            currentRecordId={record.record_id}
            tab={tab}
            canUpdate={capabilities.can_update}
            fieldMap={fieldMap}
            formValues={formValues}
            onFieldValueChange={setFieldValue}
            optionSets={schema.option_sets}
            ruleState={evaluatedRuleState}
            showTabHeader={showTabHeaders}
          />
        ))}

        {capabilities.can_update ? (
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
      {evaluatedRuleState.errorMessages.length > 0 ? (
        <Notice tone="warning">{evaluatedRuleState.errorMessages.join(" ")}</Notice>
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

  return (
    <div className="space-y-4">
      {showTabHeader ? (
        <div className="border-b border-zinc-200 pb-1">
          <p className="text-sm font-semibold text-zinc-700">{tab.display_name}</p>
        </div>
      ) : null}

      {visibleSections.map((section) => (
        <RecordSectionRenderer
          key={section.logical_name}
          appLogicalName={appLogicalName}
          currentRecordId={currentRecordId}
          section={section}
          canUpdate={canUpdate}
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
      <legend className="text-xs font-semibold uppercase tracking-[0.12em] text-zinc-500">
        {section.display_name}
      </legend>
      <div className={gridClass}>
        {columnGroups.map((columnFields, colIndex) => (
          <div key={`col-${String(colIndex)}`} className="space-y-4">
            {columnFields.map((fp) => (
              <RecordFieldControl
                key={fp.field_logical_name}
                appLogicalName={appLogicalName}
                placement={fp}
                canUpdate={canUpdate}
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
// Individual field control for record detail
// ---------------------------------------------------------------------------

type RecordFieldControlProps = {
  appLogicalName: string;
  placement: FormFieldPlacement;
  canUpdate: boolean;
  field: FieldResponse | null;
  formValues: Record<string, unknown>;
  onFieldValueChange: (fieldLogicalName: string, value: unknown) => void;
  optionSets: OptionSetResponse[];
  ruleState: ReturnType<typeof evaluateRuleState>;
};

function RecordFieldControl({
  appLogicalName,
  placement,
  canUpdate,
  field,
  formValues,
  onFieldValueChange,
  optionSets,
  ruleState,
}: RecordFieldControlProps) {
  if (!field) {
    return (
      <div className="text-xs text-zinc-400">
        Unknown field: {placement.field_logical_name}
      </div>
    );
  }

  const fieldId = `record_field_${field.logical_name}`;
  const value = formValues[field.logical_name];
  const displayLabel = placement.label_override ?? field.display_name;
  const isRequired =
    ruleState.requiredOverrides.get(field.logical_name) ??
    (placement.required_override !== null
      ? placement.required_override
      : field.is_required);
  const isReadOnly =
    (ruleState.readOnlyOverrides.get(field.logical_name) ?? placement.read_only) ||
    !canUpdate;

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

  // Boolean field
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
    const displayValue =
      typeof value === "object" && value !== null
        ? JSON.stringify(value, null, 2)
        : String(value ?? "");

    return (
      <div className="space-y-2">
        <Label htmlFor={fieldId}>
          {displayLabel}
          {isRequired ? <span className="text-red-500"> *</span> : null}
        </Label>
        <Textarea
          id={fieldId}
          className="font-mono text-xs"
          value={displayValue}
          onChange={(event) =>
            onFieldValueChange(field.logical_name, event.target.value)
          }
          placeholder='{"value":"example"}'
          readOnly={isReadOnly}
          required={isRequired}
          rows={4}
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
            <RecordFieldControl
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
              canUpdate={canUpdate}
              field={field}
              formValues={formValues}
              onFieldValueChange={onFieldValueChange}
              optionSets={optionSets}
              ruleState={ruleState}
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

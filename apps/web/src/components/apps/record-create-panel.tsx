"use client";

import { type FormEvent, useMemo, useState } from "react";
import { useRouter } from "next/navigation";

import { Label, Notice, Select } from "@qryvanta/ui";

import { evaluateRuleState } from "@/components/apps/workspace-entity/business-rules";
import { MetadataDrivenCreateForm } from "@/components/apps/workspace-entity/create-form";
import { buildFieldMap, buildInitialValues } from "@/components/apps/workspace-entity/helpers";
import type { ParsedFormResponse } from "@/components/apps/workspace-entity/metadata-types";
import {
  apiFetch,
  type AppEntityCapabilitiesResponse,
  type BusinessRuleResponse,
  type CreateRuntimeRecordRequest,
  type PublishedSchemaResponse,
  type RuntimeRecordResponse,
} from "@/lib/api";

type RecordCreatePanelProps = {
  appLogicalName: string;
  entityLogicalName: string;
  capabilities: AppEntityCapabilitiesResponse;
  schema: PublishedSchemaResponse;
  forms: ParsedFormResponse[];
  businessRules: BusinessRuleResponse[];
  initialFormLogicalName?: string | null;
  returnViewLogicalName?: string | null;
};

const AUTO_GENERATED_FIELD_NAMES = new Set([
  "record_id",
  "subject_record_id",
  "subject",
]);

export function RecordCreatePanel({
  appLogicalName,
  entityLogicalName,
  capabilities,
  schema,
  forms,
  businessRules,
  initialFormLogicalName,
  returnViewLogicalName,
}: RecordCreatePanelProps) {
  const router = useRouter();
  const fieldMap = useMemo(() => buildFieldMap(schema), [schema]);
  const [activeFormName, setActiveFormName] = useState<string | null>(
    initialFormLogicalName ??
      forms.find((form) => form.form_type === "quick_create")?.logical_name ??
      forms.find((form) => form.form_type === "main")?.logical_name ??
      forms[0]?.logical_name ??
      null,
  );
  const activeForm = useMemo(
    () => forms.find((form) => form.logical_name === activeFormName) ?? forms[0] ?? null,
    [forms, activeFormName],
  );

  const [formValues, setFormValues] = useState<Record<string, unknown>>(() =>
    buildInitialValues(schema),
  );
  const [isSaving, setIsSaving] = useState(false);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);

  const ruleState = useMemo(
    () => evaluateRuleState(businessRules, activeForm?.logical_name ?? null, formValues),
    [activeForm?.logical_name, businessRules, formValues],
  );

  async function handleCreateRecord(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();

    if (!capabilities.can_create) {
      setErrorMessage("You do not have create permission for this entity.");
      return;
    }
    if (ruleState.errorMessages.length > 0) {
      setErrorMessage(ruleState.errorMessages.join(" "));
      return;
    }

    setErrorMessage(null);
    setStatusMessage(null);
    setIsSaving(true);

    try {
      const payload: CreateRuntimeRecordRequest = {
        data: buildCreatePayload({
          activeForm,
          hiddenFieldNames: ruleState.hiddenFieldNames,
          formValues,
          schemaFields: schema.fields,
        }),
      };

      const response = await apiFetch(
        `/api/workspace/apps/${encodeURIComponent(appLogicalName)}/entities/${encodeURIComponent(entityLogicalName)}/records`,
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

      let createdRecordId: string | null = null;
      try {
        const created = (await response.json()) as RuntimeRecordResponse;
        createdRecordId = created.record_id;
      } catch {
        createdRecordId = null;
      }

      setStatusMessage("Record created.");
      if (createdRecordId) {
        const params = new URLSearchParams();
        if (activeForm?.logical_name) {
          params.set("form", activeForm.logical_name);
        }
        if (returnViewLogicalName) {
          params.set("view", returnViewLogicalName);
        }
        const suffix = params.toString() ? `?${params.toString()}` : "";
        router.replace(
          `/worker/apps/${encodeURIComponent(appLogicalName)}/${encodeURIComponent(entityLogicalName)}/${encodeURIComponent(createdRecordId)}${suffix}`,
        );
        return;
      }

      const listParams = new URLSearchParams();
      if (returnViewLogicalName) {
        listParams.set("view", returnViewLogicalName);
      }
      const listSuffix = listParams.toString() ? `?${listParams.toString()}` : "";
      router.replace(
        `/worker/apps/${encodeURIComponent(appLogicalName)}/${encodeURIComponent(entityLogicalName)}${listSuffix}`,
      );
    } catch {
      setErrorMessage("Unable to create record.");
    } finally {
      setIsSaving(false);
    }
  }

  return (
    <div className="space-y-4">
      {forms.length > 1 ? (
        <div className="rounded-lg border border-emerald-100 bg-white p-3 shadow-sm">
          <div className="space-y-1.5">
            <Label htmlFor="record_create_form_selector">
              Create Form
            </Label>
            <Select
              id="record_create_form_selector"
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

      <MetadataDrivenCreateForm
        activeForm={activeForm}
        appLogicalName={appLogicalName}
        canCreate={capabilities.can_create}
        entityDisplayName={schema.entity_display_name}
        fieldMap={fieldMap}
        formValues={formValues}
        isSaving={isSaving}
        optionSets={schema.option_sets}
        onFieldValueChange={(fieldLogicalName, value) =>
          setFormValues((current) => ({
            ...current,
            [fieldLogicalName]: value,
          }))
        }
        onSubmit={(event) => void handleCreateRecord(event)}
        ruleState={ruleState}
        schema={schema}
      />

      {errorMessage ? <Notice tone="error">{errorMessage}</Notice> : null}
      {statusMessage ? <Notice tone="success">{statusMessage}</Notice> : null}
    </div>
  );
}

function buildCreatePayload(input: {
  activeForm: ParsedFormResponse | null;
  hiddenFieldNames: Set<string>;
  formValues: Record<string, unknown>;
  schemaFields: PublishedSchemaResponse["fields"];
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
    if (AUTO_GENERATED_FIELD_NAMES.has(field.logical_name)) continue;
    const value = input.formValues[field.logical_name];
    if (input.hiddenFieldNames.has(field.logical_name)) continue;

    if (field.field_type === "boolean") {
      payload[field.logical_name] = Boolean(value);
      continue;
    }

    if (typeof value === "string") {
      const trimmed = value.trim();
      if (!trimmed) continue;

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

  const subjectField = input.schemaFields.find((field) => field.logical_name === "subject");
  if (
    subjectField?.is_required &&
    !input.hiddenFieldNames.has("subject") &&
    (payload.subject === undefined || payload.subject === null || String(payload.subject).trim() === "")
  ) {
    payload.subject = createDefaultSubject();
  }

  return payload;
}

function createDefaultSubject(): string {
  const randomPart =
    typeof crypto !== "undefined" && "randomUUID" in crypto
      ? crypto.randomUUID()
      : `${Date.now()}-${Math.floor(Math.random() * 1_000_000)}`;

  return `REC-${randomPart}`;
}

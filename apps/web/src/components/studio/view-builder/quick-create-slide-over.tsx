"use client";

import { type FormEvent, useEffect, useMemo, useState } from "react";

import { Button, Notice } from "@qryvanta/ui";

import { buildInitialValues } from "@/components/apps/workspace-entity/helpers";
import { evaluateRuleState } from "@/components/apps/workspace-entity/business-rules";
import { FieldControl } from "@/components/shared/field-control";
import { normalizeTabs } from "@/components/studio/hooks/use-form-editor-state";
import {
  apiFetch,
  type AppEntityCapabilitiesResponse,
  type BusinessRuleResponse,
  type CreateRuntimeRecordRequest,
  type FormResponse,
  type PublishedSchemaResponse,
} from "@/lib/api";

type QuickCreateSlideOverProps = {
  appLogicalName: string;
  entityLogicalName: string;
  open: boolean;
  onClose: () => void;
  onCreated: () => void;
};

type QuickCreateState = {
  schema: PublishedSchemaResponse | null;
  forms: FormResponse[];
  businessRules: BusinessRuleResponse[];
  capabilities: AppEntityCapabilitiesResponse | null;
  isLoading: boolean;
  isSaving: boolean;
  errorMessage: string | null;
  statusMessage: string | null;
};

export function QuickCreateSlideOver({
  appLogicalName,
  entityLogicalName,
  open,
  onClose,
  onCreated,
}: QuickCreateSlideOverProps) {
  const [state, setState] = useState<QuickCreateState>({
    schema: null,
    forms: [],
    businessRules: [],
    capabilities: null,
    isLoading: false,
    isSaving: false,
    errorMessage: null,
    statusMessage: null,
  });
  const [formValues, setFormValues] = useState<Record<string, unknown>>({});

  useEffect(() => {
    if (!open) return;
    let isMounted = true;

    async function load(): Promise<void> {
      setState((current) => ({
        ...current,
        isLoading: true,
        errorMessage: null,
        statusMessage: null,
      }));
      try {
        const [schemaResponse, formsResponse, rulesResponse, capabilitiesResponse] = await Promise.all([
          apiFetch(
            `/api/workspace/apps/${encodeURIComponent(appLogicalName)}/entities/${encodeURIComponent(entityLogicalName)}/schema`,
          ),
          apiFetch(
            `/api/workspace/apps/${encodeURIComponent(appLogicalName)}/entities/${encodeURIComponent(entityLogicalName)}/forms`,
          ),
          apiFetch(`/api/runtime/${encodeURIComponent(entityLogicalName)}/business-rules`),
          apiFetch(
            `/api/workspace/apps/${encodeURIComponent(appLogicalName)}/entities/${encodeURIComponent(entityLogicalName)}/capabilities`,
          ),
        ]);

        if (!isMounted) return;
        if (!schemaResponse.ok || !formsResponse.ok || !capabilitiesResponse.ok) {
          setState((current) => ({
            ...current,
            isLoading: false,
            errorMessage: "Unable to load quick create metadata.",
          }));
          return;
        }

        const schema = (await schemaResponse.json()) as PublishedSchemaResponse;
        const forms = (await formsResponse.json()) as FormResponse[];
        const capabilities = (await capabilitiesResponse.json()) as AppEntityCapabilitiesResponse;
        const businessRules = rulesResponse.ok
          ? ((await rulesResponse.json()) as BusinessRuleResponse[])
          : [];

        setState({
          schema,
          forms,
          businessRules,
          capabilities,
          isLoading: false,
          isSaving: false,
          errorMessage: null,
          statusMessage: null,
        });
        setFormValues(buildInitialValues(schema));
      } catch {
        if (isMounted) {
          setState((current) => ({
            ...current,
            isLoading: false,
            errorMessage: "Unable to load quick create metadata.",
          }));
        }
      }
    }

    void load();

    return () => {
      isMounted = false;
    };
  }, [appLogicalName, entityLogicalName, open]);

  const activeForm = useMemo(() => {
    if (state.forms.length === 0) return null;
    return (
      state.forms.find((form) => form.form_type === "quick_create") ??
      state.forms.find((form) => form.form_type === "main") ??
      state.forms[0]
    );
  }, [state.forms]);

  const tabs = useMemo(() => normalizeTabs(activeForm?.tabs), [activeForm?.tabs]);
  const fieldMap = useMemo(
    () => new Map((state.schema?.fields ?? []).map((field) => [field.logical_name, field])),
    [state.schema?.fields],
  );

  const ruleState = useMemo(
    () => evaluateRuleState(state.businessRules, activeForm?.logical_name ?? null, formValues),
    [activeForm?.logical_name, formValues, state.businessRules],
  );

  if (!open) return null;

  async function handleCreate(event: FormEvent<HTMLFormElement>): Promise<void> {
    event.preventDefault();
    if (!state.schema || !state.capabilities) return;

    if (!state.capabilities.can_create) {
      setState((current) => ({
        ...current,
        errorMessage: "You do not have create permission for this entity.",
      }));
      return;
    }

    setState((current) => ({
      ...current,
      isSaving: true,
      errorMessage: null,
      statusMessage: null,
    }));

    try {
      const payload: CreateRuntimeRecordRequest = {
        data: buildCreatePayload({
          tabs,
          schemaFields: state.schema.fields,
          hiddenFieldNames: ruleState.hiddenFieldNames,
          formValues,
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
        setState((current) => ({
          ...current,
          isSaving: false,
          errorMessage: body.message ?? "Unable to create record.",
        }));
        return;
      }

      setState((current) => ({
        ...current,
        isSaving: false,
        statusMessage: "Record created.",
      }));
      onCreated();
      onClose();
    } catch {
      setState((current) => ({
        ...current,
        isSaving: false,
        errorMessage: "Unable to create record.",
      }));
    }
  }

  return (
    <div className="fixed inset-0 z-50 flex">
      <button
        type="button"
        className="flex-1 bg-black/20"
        aria-label="Close quick create"
        onClick={onClose}
      />
      <div className="h-full w-full max-w-3xl overflow-y-auto border-l border-zinc-200 bg-white p-4">
        <div className="mb-3 flex items-center justify-between">
          <p className="font-serif text-xl text-zinc-900">Quick Create</p>
          <Button type="button" variant="outline" size="sm" onClick={onClose}>
            Close
          </Button>
        </div>

        {state.isLoading || !state.schema ? (
          <p className="text-sm text-zinc-500">Loading quick create form...</p>
        ) : (
          <form className="space-y-4" onSubmit={(event) => void handleCreate(event)}>
            {tabs.map((tab) => (
              <div key={tab.logical_name} className="space-y-3">
                {tab.sections
                  .filter((section) => section.visible)
                  .sort((a, b) => a.position - b.position)
                  .map((section) => {
                    const fields = section.fields
                      .filter((placement) => placement.visible)
                      .sort((a, b) => a.position - b.position);
                    return (
                      <div
                        key={`${tab.logical_name}-${section.logical_name}`}
                        className="rounded-md border border-zinc-200 p-3"
                      >
                        <p className="mb-2 text-xs font-semibold uppercase tracking-[0.12em] text-zinc-600">
                          {section.display_name}
                        </p>
                        <div
                          className={
                            section.columns === 1
                              ? "grid gap-3"
                              : section.columns === 2
                                ? "grid gap-3 md:grid-cols-2"
                                : "grid gap-3 md:grid-cols-3"
                          }
                        >
                          {Array.from({ length: section.columns }).map((_, columnIndex) => (
                            <div key={`quick-column-${columnIndex}`} className="space-y-3">
                              {fields
                                .filter((placement) => placement.column === columnIndex)
                                .map((placement) => (
                                  <FieldControl
                                    key={`${placement.field_logical_name}-${placement.position}`}
                                    appLogicalName={appLogicalName}
                                    placement={placement}
                                    field={fieldMap.get(placement.field_logical_name) ?? null}
                                    formValues={formValues}
                                    onFieldValueChange={(fieldLogicalName, value) =>
                                      setFormValues((current) => ({
                                        ...current,
                                        [fieldLogicalName]: value,
                                      }))
                                    }
                                    optionSets={state.schema?.option_sets ?? []}
                                    ruleState={ruleState}
                                    canEdit
                                    fieldIdPrefix="studio_quick_create"
                                  />
                                ))}
                            </div>
                          ))}
                        </div>
                      </div>
                    );
                  })}
              </div>
            ))}

            <div className="flex items-center justify-end gap-2 border-t border-zinc-200 pt-3">
              <Button type="button" variant="outline" size="sm" onClick={onClose}>
                Cancel
              </Button>
              <Button type="submit" size="sm" disabled={state.isSaving}>
                {state.isSaving ? "Creating..." : "Create Record"}
              </Button>
            </div>
          </form>
        )}

        {state.errorMessage ? <Notice tone="error">{state.errorMessage}</Notice> : null}
        {state.statusMessage ? <Notice tone="success">{state.statusMessage}</Notice> : null}
      </div>
    </div>
  );
}

function buildCreatePayload(input: {
  tabs: ReturnType<typeof normalizeTabs>;
  schemaFields: PublishedSchemaResponse["fields"];
  hiddenFieldNames: Set<string>;
  formValues: Record<string, unknown>;
}): Record<string, unknown> {
  const payload: Record<string, unknown> = {};

  const formFieldNames = new Set<string>();
  for (const tab of input.tabs) {
    for (const section of tab.sections) {
      for (const placement of section.fields) {
        formFieldNames.add(placement.field_logical_name);
      }
    }
  }

  const fieldsToProcess =
    formFieldNames.size > 0
      ? input.schemaFields.filter((field) => formFieldNames.has(field.logical_name))
      : input.schemaFields;

  for (const field of fieldsToProcess) {
    if (input.hiddenFieldNames.has(field.logical_name)) continue;
    const value = input.formValues[field.logical_name];

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

  return payload;
}

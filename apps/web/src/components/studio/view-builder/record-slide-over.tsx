"use client";

import { useEffect, useState } from "react";

import { Button } from "@qryvanta/ui";

import { RecordDetailPanel } from "@/components/apps/record-detail-panel";
import { parseFormResponse } from "@/components/apps/workspace-entity/helpers";
import {
  apiFetch,
  type AppEntityCapabilitiesResponse,
  type BusinessRuleResponse,
  type FormResponse,
  type PublishedSchemaResponse,
  type RuntimeRecordResponse,
} from "@/lib/api";

type RecordSlideOverProps = {
  appLogicalName: string;
  entityLogicalName: string;
  record: RuntimeRecordResponse | null;
  onClose: () => void;
};

type DetailState = {
  capabilities: AppEntityCapabilitiesResponse | null;
  schema: PublishedSchemaResponse | null;
  forms: FormResponse[];
  businessRules: BusinessRuleResponse[];
  loadedRecord: RuntimeRecordResponse | null;
  isLoading: boolean;
};

export function RecordSlideOver({
  appLogicalName,
  entityLogicalName,
  record,
  onClose,
}: RecordSlideOverProps) {
  const [state, setState] = useState<DetailState>({
    capabilities: null,
    schema: null,
    forms: [],
    businessRules: [],
    loadedRecord: null,
    isLoading: false,
  });

  useEffect(() => {
    if (!record) return;
    const recordId = record.record_id;
    let isMounted = true;

    async function load(): Promise<void> {
      setState((current) => ({ ...current, isLoading: true }));
      try {
        const [schemaResponse, capabilitiesResponse, recordResponse, formsResponse, rulesResponse] =
          await Promise.all([
            apiFetch(
              `/api/workspace/apps/${encodeURIComponent(appLogicalName)}/entities/${encodeURIComponent(entityLogicalName)}/schema`,
            ),
            apiFetch(
              `/api/workspace/apps/${encodeURIComponent(appLogicalName)}/entities/${encodeURIComponent(entityLogicalName)}/capabilities`,
            ),
            apiFetch(
              `/api/workspace/apps/${encodeURIComponent(appLogicalName)}/entities/${encodeURIComponent(entityLogicalName)}/records/${encodeURIComponent(recordId)}`,
            ),
            apiFetch(
              `/api/workspace/apps/${encodeURIComponent(appLogicalName)}/entities/${encodeURIComponent(entityLogicalName)}/forms`,
            ),
            apiFetch(`/api/runtime/${encodeURIComponent(entityLogicalName)}/business-rules`),
          ]);

        if (!isMounted) return;
        if (
          !schemaResponse.ok ||
          !capabilitiesResponse.ok ||
          !recordResponse.ok ||
          !formsResponse.ok
        ) {
          setState((current) => ({ ...current, isLoading: false }));
          return;
        }

        setState({
          capabilities: (await capabilitiesResponse.json()) as AppEntityCapabilitiesResponse,
          schema: (await schemaResponse.json()) as PublishedSchemaResponse,
          forms: (await formsResponse.json()) as FormResponse[],
          businessRules: rulesResponse.ok
            ? ((await rulesResponse.json()) as BusinessRuleResponse[])
            : [],
          loadedRecord: (await recordResponse.json()) as RuntimeRecordResponse,
          isLoading: false,
        });
      } catch {
        if (isMounted) {
          setState((current) => ({ ...current, isLoading: false }));
        }
      }
    }

    void load();

    return () => {
      isMounted = false;
    };
  }, [appLogicalName, entityLogicalName, record]);

  if (!record) return null;

  return (
    <div className="fixed inset-0 z-50 flex">
      <button
        type="button"
        className="flex-1 bg-black/20"
        aria-label="Close record preview"
        onClick={onClose}
      />
      <div className="h-full w-full max-w-3xl overflow-y-auto border-l border-zinc-200 bg-white p-4">
        <div className="mb-4 flex items-center justify-between">
          <p className="font-serif text-xl text-zinc-900">Record preview</p>
          <Button type="button" variant="outline" size="sm" onClick={onClose}>
            Close
          </Button>
        </div>

        {state.isLoading || !state.schema || !state.capabilities || !state.loadedRecord ? (
          <p className="text-sm text-zinc-500">Loading record detail...</p>
        ) : (
          <RecordDetailPanel
            appLogicalName={appLogicalName}
            entityLogicalName={entityLogicalName}
            capabilities={state.capabilities}
            forms={state.forms.map(parseFormResponse)}
            businessRules={state.businessRules}
            initialFormLogicalName={null}
            record={state.loadedRecord}
            schema={state.schema}
          />
        )}
      </div>
    </div>
  );
}

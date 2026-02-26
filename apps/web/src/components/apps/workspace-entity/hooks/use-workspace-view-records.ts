import { useEffect, useMemo, useState } from "react";

import {
  apiFetch,
  type QueryRuntimeRecordsRequest,
  type RuntimeRecordResponse,
} from "@/lib/api";
import type { ParsedViewResponse } from "@/components/apps/workspace-entity/metadata-types";

type UseWorkspaceViewRecordsInput = {
  appLogicalName: string;
  entityLogicalName: string;
  activeView: ParsedViewResponse | null;
  records: RuntimeRecordResponse[];
};

type UseWorkspaceViewRecordsResult = {
  runtimeRecords: RuntimeRecordResponse[];
  isRefreshingRecords: boolean;
  refreshErrorMessage: string | null;
};

type RecordLoadState = {
  isRefreshingRecords: boolean;
  fetchedRecords: RuntimeRecordResponse[];
  refreshErrorMessage: string | null;
};

export function useWorkspaceViewRecords({
  appLogicalName,
  entityLogicalName,
  activeView,
  records,
}: UseWorkspaceViewRecordsInput): UseWorkspaceViewRecordsResult {
  const [recordLoadState, setRecordLoadState] = useState<RecordLoadState>({
    isRefreshingRecords: false,
    fetchedRecords: [],
    refreshErrorMessage: null,
  });

  useEffect(() => {
    let cancelled = false;

    function updateState(next: RecordLoadState | ((current: RecordLoadState) => RecordLoadState)) {
      if (cancelled) {
        return;
      }
      setRecordLoadState(next);
    }

    async function refreshRecordsForActiveView() {
      if (!activeView) {
        updateState({
          isRefreshingRecords: false,
          fetchedRecords: [],
          refreshErrorMessage: null,
        });
        return;
      }

      updateState((current) => ({
        ...current,
        isRefreshingRecords: true,
        refreshErrorMessage: null,
      }));

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
          updateState((current) => ({
            ...current,
            isRefreshingRecords: false,
            refreshErrorMessage:
              body.message ?? "Unable to refresh records for selected view.",
          }));
          return;
        }

        const nextRecords = (await response.json()) as RuntimeRecordResponse[];
        updateState({
          isRefreshingRecords: false,
          fetchedRecords: nextRecords,
          refreshErrorMessage: null,
        });
      } catch {
        updateState((current) => ({
          ...current,
          isRefreshingRecords: false,
          refreshErrorMessage: "Unable to refresh records for selected view.",
        }));
      }
    }

    void refreshRecordsForActiveView();

    return () => {
      cancelled = true;
    };
  }, [activeView, appLogicalName, entityLogicalName]);

  const runtimeRecords = useMemo(
    () => (activeView ? recordLoadState.fetchedRecords : records),
    [activeView, recordLoadState.fetchedRecords, records],
  );

  return {
    runtimeRecords,
    isRefreshingRecords: recordLoadState.isRefreshingRecords,
    refreshErrorMessage: recordLoadState.refreshErrorMessage,
  };
}

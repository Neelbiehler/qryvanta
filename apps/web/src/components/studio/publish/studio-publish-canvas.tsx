"use client";

import { useEffect, useMemo, useState } from "react";

import { Button, Notice } from "@qryvanta/ui";

import { PublishDiffPanel } from "@/components/studio/publish/publish-diff-panel";
import { PublishHistoryPanel } from "@/components/studio/publish/publish-history-panel";
import { WorkspaceChecksPanel } from "@/components/studio/publish/workspace-checks-panel";
import {
  apiFetch,
  type AppPublishChecksResponse,
  type AppResponse,
  type EntityResponse,
  type RunWorkspacePublishRequest,
  type RunWorkspacePublishResponse,
  type WorkspacePublishChecksResponse,
  type WorkspacePublishDiffRequest,
  type WorkspacePublishDiffResponse,
  type WorkspacePublishHistoryEntryResponse,
} from "@/lib/api";

type StudioPublishCanvasProps = {
  apps: AppResponse[];
  entities: EntityResponse[];
  selectedApp: string;
};

type WorkspacePublishDraft = {
  entityLogicalNames: string[];
  appLogicalNames: string[];
};

type SelectionValidationState = {
  selectionKey: string | null;
  isPublishable: boolean;
};

type PublishRunHistoryEntry = {
  runId: string;
  runAt: string;
  subject: string;
  requestedEntities: number;
  requestedApps: number;
  requestedEntityLogicalNames: string[];
  requestedAppLogicalNames: string[];
  publishedEntities: string[];
  validatedApps: string[];
  issueCount: number;
  isPublishable: boolean;
};

export function StudioPublishCanvas({ apps, entities, selectedApp }: StudioPublishCanvasProps) {
  const [isRunningPublishChecks, setIsRunningPublishChecks] = useState(false);
  const [publishCheckErrors, setPublishCheckErrors] = useState<string[]>([]);
  const [isRunningWorkspaceChecks, setIsRunningWorkspaceChecks] = useState(false);
  const [isRunningSelectionChecks, setIsRunningSelectionChecks] = useState(false);
  const [isRunningSelectivePublish, setIsRunningSelectivePublish] = useState(false);
  const [workspaceIssues, setWorkspaceIssues] = useState<WorkspacePublishChecksResponse["issues"]>([]);
  const [workspaceCheckSummary, setWorkspaceCheckSummary] = useState<{
    checkedEntities: number;
    checkedApps: number;
  } | null>(null);
  const [workspacePublishDraft, setWorkspacePublishDraft] = useState<WorkspacePublishDraft>({
    entityLogicalNames: entities.map((entity) => entity.logical_name),
    appLogicalNames: apps.map((app) => app.logical_name),
  });
  const [selectionValidation, setSelectionValidation] = useState<SelectionValidationState>({
    selectionKey: null,
    isPublishable: false,
  });
  const [publishHistory, setPublishHistory] = useState<PublishRunHistoryEntry[]>([]);
  const [publishDiff, setPublishDiff] = useState<{
    unknownEntityLogicalNames: string[];
    unknownAppLogicalNames: string[];
    entityDiffs: WorkspacePublishDiffResponse["entity_diffs"];
    appDiffs: WorkspacePublishDiffResponse["app_diffs"];
  }>({
    unknownEntityLogicalNames: [],
    unknownAppLogicalNames: [],
    entityDiffs: [],
    appDiffs: [],
  });
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);

  const workspaceSelectionKey = useMemo(
    () =>
      JSON.stringify({
        entityLogicalNames: [...workspacePublishDraft.entityLogicalNames].sort(),
        appLogicalNames: [...workspacePublishDraft.appLogicalNames].sort(),
      }),
    [workspacePublishDraft.appLogicalNames, workspacePublishDraft.entityLogicalNames],
  );

  useEffect(() => {
    setWorkspacePublishDraft((current) => ({
      entityLogicalNames: reconcileSelections(
        current.entityLogicalNames,
        entities.map((entity) => entity.logical_name),
      ),
      appLogicalNames: reconcileSelections(
        current.appLogicalNames,
        apps.map((app) => app.logical_name),
      ),
    }));
  }, [apps, entities]);

  useEffect(() => {
    setSelectionValidation((current) =>
      current.selectionKey === workspaceSelectionKey
        ? current
        : { selectionKey: null, isPublishable: false },
    );
  }, [workspaceSelectionKey]);

  useEffect(() => {
    void refreshPublishHistory();
    void refreshPublishDiff();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  async function refreshPublishHistory(): Promise<void> {
    try {
      const response = await apiFetch("/api/publish/history?limit=12");
      if (!response.ok) {
        setPublishHistory([]);
        return;
      }

      const entries = (await response.json()) as WorkspacePublishHistoryEntryResponse[];
      setPublishHistory(
        entries.map((entry) => ({
          runId: entry.run_id,
          runAt: entry.run_at,
          subject: entry.subject,
          requestedEntities: entry.requested_entities,
          requestedApps: entry.requested_apps,
          requestedEntityLogicalNames: entry.requested_entity_logical_names,
          requestedAppLogicalNames: entry.requested_app_logical_names,
          publishedEntities: entry.published_entities,
          validatedApps: entry.validated_apps,
          issueCount: entry.issue_count,
          isPublishable: entry.is_publishable,
        })),
      );
    } catch {
      setPublishHistory([]);
    }
  }

  async function refreshPublishDiff(): Promise<void> {
    try {
      const payload: WorkspacePublishDiffRequest = {
        entity_logical_names: workspacePublishDraft.entityLogicalNames,
        app_logical_names: workspacePublishDraft.appLogicalNames,
      };

      const response = await apiFetch("/api/publish/diff", {
        method: "POST",
        body: JSON.stringify(payload),
      });
      if (!response.ok) {
        setPublishDiff({
          unknownEntityLogicalNames: [],
          unknownAppLogicalNames: [],
          entityDiffs: [],
          appDiffs: [],
        });
        return;
      }

      const result = (await response.json()) as WorkspacePublishDiffResponse;
      setPublishDiff({
        unknownEntityLogicalNames: result.unknown_entity_logical_names,
        unknownAppLogicalNames: result.unknown_app_logical_names,
        entityDiffs: result.entity_diffs,
        appDiffs: result.app_diffs,
      });
    } catch {
      setPublishDiff({
        unknownEntityLogicalNames: [],
        unknownAppLogicalNames: [],
        entityDiffs: [],
        appDiffs: [],
      });
    }
  }

  async function handleRunPublishChecks(): Promise<void> {
    if (!selectedApp) {
      setErrorMessage("Select an app first.");
      return;
    }

    setErrorMessage(null);
    setStatusMessage(null);
    setIsRunningPublishChecks(true);

    try {
      const response = await apiFetch(`/api/apps/${encodeURIComponent(selectedApp)}/publish-checks`);
      if (!response.ok) {
        const payload = (await response.json()) as { message?: string };
        setErrorMessage(payload.message ?? "Unable to run app publish checks.");
        return;
      }

      const payload = (await response.json()) as AppPublishChecksResponse;
      setPublishCheckErrors(payload.errors);
      if (payload.is_publishable) {
        setStatusMessage("App publish checks passed.");
      } else {
        setErrorMessage("App publish checks found issues.");
      }
    } catch {
      setErrorMessage("Unable to run app publish checks.");
    } finally {
      setIsRunningPublishChecks(false);
    }
  }

  async function handleRunWorkspaceChecks(): Promise<void> {
    setErrorMessage(null);
    setStatusMessage(null);
    setIsRunningWorkspaceChecks(true);
    setPublishCheckErrors([]);

    try {
      const response = await apiFetch("/api/publish/checks");
      if (!response.ok) {
        const payload = (await response.json()) as { message?: string };
        setErrorMessage(payload.message ?? "Unable to run workspace publish checks.");
        return;
      }

      const payload = (await response.json()) as WorkspacePublishChecksResponse;
      setWorkspaceIssues(payload.issues);
      setWorkspaceCheckSummary({
        checkedEntities: payload.checked_entities,
        checkedApps: payload.checked_apps,
      });
      setStatusMessage(payload.is_publishable ? "Workspace checks passed." : "Workspace checks found issues.");
      await refreshPublishDiff();
    } catch {
      setErrorMessage("Unable to run workspace publish checks.");
    } finally {
      setIsRunningWorkspaceChecks(false);
    }
  }

  async function handleRunSelectionChecks(): Promise<void> {
    setErrorMessage(null);
    setStatusMessage(null);
    setIsRunningSelectionChecks(true);
    setPublishCheckErrors([]);

    try {
      const payload: RunWorkspacePublishRequest = {
        entity_logical_names: workspacePublishDraft.entityLogicalNames,
        app_logical_names: workspacePublishDraft.appLogicalNames,
        dry_run: true,
      };

      const response = await apiFetch("/api/publish/checks", {
        method: "POST",
        body: JSON.stringify(payload),
      });
      if (!response.ok) {
        const body = (await response.json()) as { message?: string };
        setErrorMessage(body.message ?? "Unable to validate selection.");
        return;
      }

      const result = (await response.json()) as RunWorkspacePublishResponse;
      setWorkspaceIssues(result.issues);
      setWorkspaceCheckSummary({
        checkedEntities: result.requested_entities,
        checkedApps: result.requested_apps,
      });
      setSelectionValidation({
        selectionKey: workspaceSelectionKey,
        isPublishable: result.is_publishable,
      });
      setStatusMessage(result.is_publishable ? "Selection checks passed." : "Selection checks found issues.");
      await refreshPublishDiff();
    } catch {
      setErrorMessage("Unable to validate selection.");
    } finally {
      setIsRunningSelectionChecks(false);
    }
  }

  async function handleRunSelectivePublish(): Promise<void> {
    if (selectionValidation.selectionKey !== workspaceSelectionKey) {
      setErrorMessage("Run selection checks before publishing.");
      return;
    }
    if (!selectionValidation.isPublishable) {
      setErrorMessage("Selection is not publishable.");
      return;
    }

    setErrorMessage(null);
    setStatusMessage(null);
    setIsRunningSelectivePublish(true);
    setPublishCheckErrors([]);

    try {
      const payload: RunWorkspacePublishRequest = {
        entity_logical_names: workspacePublishDraft.entityLogicalNames,
        app_logical_names: workspacePublishDraft.appLogicalNames,
        dry_run: false,
      };

      const response = await apiFetch("/api/publish/checks", {
        method: "POST",
        body: JSON.stringify(payload),
      });
      if (!response.ok) {
        const body = (await response.json()) as { message?: string };
        setErrorMessage(body.message ?? "Unable to run selective publish.");
        return;
      }

      const result = (await response.json()) as RunWorkspacePublishResponse;
      setWorkspaceIssues(result.issues);
      setWorkspaceCheckSummary({
        checkedEntities: result.requested_entities,
        checkedApps: result.requested_apps,
      });
      setSelectionValidation({
        selectionKey: workspaceSelectionKey,
        isPublishable: result.is_publishable,
      });

      await refreshPublishHistory();
      await refreshPublishDiff();

      if (result.is_publishable) {
        setStatusMessage(
          `Selective publish complete: ${result.published_entities.length} entities, ${result.validated_apps.length} apps.`,
        );
      } else {
        setErrorMessage("Selective publish blocked by publish issues.");
      }
    } catch {
      setErrorMessage("Unable to run selective publish.");
    } finally {
      setIsRunningSelectivePublish(false);
    }
  }

  return (
    <div className="space-y-3 rounded-xl border border-zinc-200 bg-zinc-50 p-3">
      <div className="rounded-md border border-zinc-200 bg-white p-3">
        <p className="text-xs font-semibold uppercase tracking-[0.14em] text-zinc-600">
          Publish Controls
        </p>
        <div className="mt-2 grid gap-3 xl:grid-cols-2">
          <div className="space-y-2">
            <p className="text-xs font-semibold text-zinc-700">Entity Selection</p>
            <div className="max-h-44 space-y-1 overflow-y-auto rounded border border-zinc-200 p-2">
              {entities.map((entity) => {
                const checked = workspacePublishDraft.entityLogicalNames.includes(entity.logical_name);
                return (
                  <label key={entity.logical_name} className="flex items-center gap-2 text-xs">
                    <input
                      type="checkbox"
                      checked={checked}
                      onChange={(event) => {
                        const next = event.target.checked
                          ? [...workspacePublishDraft.entityLogicalNames, entity.logical_name]
                          : workspacePublishDraft.entityLogicalNames.filter(
                              (name) => name !== entity.logical_name,
                            );
                        setWorkspacePublishDraft((current) => ({
                          ...current,
                          entityLogicalNames: next,
                        }));
                      }}
                    />
                    {entity.display_name} ({entity.logical_name})
                  </label>
                );
              })}
            </div>
          </div>

          <div className="space-y-2">
            <p className="text-xs font-semibold text-zinc-700">App Selection</p>
            <div className="max-h-44 space-y-1 overflow-y-auto rounded border border-zinc-200 p-2">
              {apps.map((app) => {
                const checked = workspacePublishDraft.appLogicalNames.includes(app.logical_name);
                return (
                  <label key={app.logical_name} className="flex items-center gap-2 text-xs">
                    <input
                      type="checkbox"
                      checked={checked}
                      onChange={(event) => {
                        const next = event.target.checked
                          ? [...workspacePublishDraft.appLogicalNames, app.logical_name]
                          : workspacePublishDraft.appLogicalNames.filter(
                              (name) => name !== app.logical_name,
                            );
                        setWorkspacePublishDraft((current) => ({
                          ...current,
                          appLogicalNames: next,
                        }));
                      }}
                    />
                    {app.display_name} ({app.logical_name})
                  </label>
                );
              })}
            </div>
          </div>
        </div>

        <div className="mt-3 flex flex-wrap items-center gap-2">
          <Button
            type="button"
            variant="outline"
            onClick={() => void handleRunPublishChecks()}
            disabled={!selectedApp || isRunningPublishChecks}
          >
            {isRunningPublishChecks ? "Running app checks..." : "Run App Publish Checks"}
          </Button>
          <Button
            type="button"
            variant="outline"
            onClick={() => void handleRunWorkspaceChecks()}
            disabled={isRunningWorkspaceChecks}
          >
            {isRunningWorkspaceChecks ? "Running workspace checks..." : "Run Workspace Checks"}
          </Button>
          <Button
            type="button"
            variant="outline"
            onClick={() => void handleRunSelectionChecks()}
            disabled={isRunningSelectionChecks}
          >
            {isRunningSelectionChecks ? "Validating selection..." : "Run Selection Checks"}
          </Button>
          <Button
            type="button"
            onClick={() => void handleRunSelectivePublish()}
            disabled={
              isRunningSelectivePublish ||
              selectionValidation.selectionKey !== workspaceSelectionKey ||
              !selectionValidation.isPublishable
            }
          >
            {isRunningSelectivePublish ? "Publishing..." : "Publish Selection"}
          </Button>
        </div>

        {workspaceCheckSummary ? (
          <p className="mt-2 text-xs text-zinc-600">
            Checked {workspaceCheckSummary.checkedEntities} entities / {workspaceCheckSummary.checkedApps} apps
          </p>
        ) : null}
      </div>

      <WorkspaceChecksPanel
        publishCheckErrors={publishCheckErrors}
        workspaceIssues={workspaceIssues}
      />

      <PublishHistoryPanel
        publishHistory={publishHistory}
        onLoadSelection={(entityLogicalNames, appLogicalNames) => {
          setWorkspacePublishDraft({
            entityLogicalNames: reconcileSelections(
              entityLogicalNames,
              entities.map((entity) => entity.logical_name),
            ),
            appLogicalNames: reconcileSelections(
              appLogicalNames,
              apps.map((app) => app.logical_name),
            ),
          });
        }}
      />

      <PublishDiffPanel publishDiff={publishDiff} />

      {errorMessage ? <Notice tone="error">{errorMessage}</Notice> : null}
      {statusMessage ? <Notice tone="success">{statusMessage}</Notice> : null}
    </div>
  );
}

function reconcileSelections(selected: string[], available: string[]): string[] {
  const availableSet = new Set(available);
  const retained = selected.filter((value) => availableSet.has(value));
  if (retained.length > 0) {
    return retained;
  }
  return available;
}

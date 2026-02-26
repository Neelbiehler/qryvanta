import { useState } from "react";

import type {
  AppPublishDiffResponse,
  EntityPublishDiffResponse,
  PublishCheckIssueResponse,
} from "@/lib/api";
import type { AppResponse, EntityResponse } from "@/lib/api";

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

type UseWorkspacePublishInput = {
  entities: EntityResponse[];
  apps: AppResponse[];
};

export function useWorkspacePublish({ entities, apps }: UseWorkspacePublishInput) {
  const [isRunningPublishChecks, setIsRunningPublishChecks] = useState(false);
  const [publishCheckErrors, setPublishCheckErrors] = useState<string[]>([]);
  const [isRunningWorkspaceChecks, setIsRunningWorkspaceChecks] = useState(false);
  const [isRunningSelectionChecks, setIsRunningSelectionChecks] = useState(false);
  const [workspaceIssues, setWorkspaceIssues] = useState<PublishCheckIssueResponse[]>([]);
  const [workspaceCheckSummary, setWorkspaceCheckSummary] = useState<{
    checkedEntities: number;
    checkedApps: number;
  } | null>(null);
  const [workspacePublishDraft, setWorkspacePublishDraft] =
    useState<WorkspacePublishDraft>({
      entityLogicalNames: entities.map((entity) => entity.logical_name),
      appLogicalNames: apps.map((app) => app.logical_name),
    });
  const [isRunningSelectivePublish, setIsRunningSelectivePublish] = useState(false);
  const [selectionValidation, setSelectionValidation] =
    useState<SelectionValidationState>({
      selectionKey: null,
      isPublishable: false,
    });
  const [publishHistory, setPublishHistory] = useState<PublishRunHistoryEntry[]>([]);
  const [publishDiff, setPublishDiff] = useState<{
    unknownEntityLogicalNames: string[];
    unknownAppLogicalNames: string[];
    entityDiffs: EntityPublishDiffResponse[];
    appDiffs: AppPublishDiffResponse[];
  }>({
    unknownEntityLogicalNames: [],
    unknownAppLogicalNames: [],
    entityDiffs: [],
    appDiffs: [],
  });

  return {
    isRunningPublishChecks,
    setIsRunningPublishChecks,
    publishCheckErrors,
    setPublishCheckErrors,
    isRunningWorkspaceChecks,
    setIsRunningWorkspaceChecks,
    isRunningSelectionChecks,
    setIsRunningSelectionChecks,
    workspaceIssues,
    setWorkspaceIssues,
    workspaceCheckSummary,
    setWorkspaceCheckSummary,
    workspacePublishDraft,
    setWorkspacePublishDraft,
    isRunningSelectivePublish,
    setIsRunningSelectivePublish,
    selectionValidation,
    setSelectionValidation,
    publishHistory,
    setPublishHistory,
    publishDiff,
    setPublishDiff,
  };
}

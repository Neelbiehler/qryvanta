import { useCallback, useRef, useState } from "react";

import type {
  CatalogInsertMode,
  FlowTemplateCategory,
  InspectorNode,
} from "@/components/automation/workflow-studio/model";
import type { WorkflowResponse } from "@/lib/api";

export type WorkflowWorkspaceMode = "edit" | "history";

type WorkflowWorkspaceState = {
  selectedWorkflow: string;
  workflowWorkspaceMode: WorkflowWorkspaceMode;
  workflowQuery: string;
  errorMessage: string | null;
  statusMessage: string | null;
};

type WorkflowCatalogState = {
  catalogQuery: string;
  catalogCategory: "all" | FlowTemplateCategory;
  catalogInsertMode: CatalogInsertMode;
};

type WorkflowStudioStateArgs = {
  workflows: WorkflowResponse[];
  initialSelectedWorkflow?: string;
  initialWorkspaceMode?: WorkflowWorkspaceMode;
};

export function useWorkflowStudioState({
  workflows,
  initialSelectedWorkflow,
  initialWorkspaceMode,
}: WorkflowStudioStateArgs) {
  const idCounterRef = useRef(1);

  const createId = useCallback((): string => {
    const id = `flow_step_${idCounterRef.current}`;
    idCounterRef.current += 1;
    return id;
  }, []);

  const initialWorkflowSelection =
    initialSelectedWorkflow === undefined
      ? workflows.at(0)?.logical_name ?? ""
      : workflows.some((workflow) => workflow.logical_name === initialSelectedWorkflow)
        ? initialSelectedWorkflow
        : "";

  const [workspaceState, setWorkspaceState] = useState<WorkflowWorkspaceState>({
    selectedWorkflow: initialWorkflowSelection,
    workflowWorkspaceMode: initialWorkspaceMode ?? "edit",
    workflowQuery: "",
    errorMessage: null,
    statusMessage: null,
  });

  const [catalogState, setCatalogState] = useState<WorkflowCatalogState>({
    catalogQuery: "",
    catalogCategory: "all",
    catalogInsertMode: "after_selected",
  });

  const [selectionState, setSelectionState] = useState<{
    inspectorNode: InspectorNode;
    selectedStepId: string | null;
  }>({
    inspectorNode: "trigger",
    selectedStepId: null,
  });

  const setSelectedWorkflow = useCallback((next: string) => {
    setWorkspaceState((current) => ({ ...current, selectedWorkflow: next }));
  }, []);

  const setWorkflowWorkspaceMode = useCallback((next: WorkflowWorkspaceMode) => {
    setWorkspaceState((current) => ({ ...current, workflowWorkspaceMode: next }));
  }, []);

  const setWorkflowQuery = useCallback((next: string) => {
    setWorkspaceState((current) => ({ ...current, workflowQuery: next }));
  }, []);

  const setErrorMessage = useCallback((next: string | null) => {
    setWorkspaceState((current) => ({ ...current, errorMessage: next }));
  }, []);

  const setStatusMessage = useCallback((next: string | null) => {
    setWorkspaceState((current) => ({ ...current, statusMessage: next }));
  }, []);

  const setCatalogQuery = useCallback((next: string) => {
    setCatalogState((current) => ({ ...current, catalogQuery: next }));
  }, []);

  const setCatalogCategory = useCallback((next: "all" | FlowTemplateCategory) => {
    setCatalogState((current) => ({ ...current, catalogCategory: next }));
  }, []);

  const setCatalogInsertMode = useCallback((next: CatalogInsertMode) => {
    setCatalogState((current) => ({ ...current, catalogInsertMode: next }));
  }, []);

  const setInspectorNode = useCallback((next: InspectorNode) => {
    setSelectionState((current) => ({ ...current, inspectorNode: next }));
  }, []);

  const setSelectedStepId = useCallback((next: string | null) => {
    setSelectionState((current) => ({ ...current, selectedStepId: next }));
  }, []);

  return {
    createId,
    workspaceState,
    setWorkspaceState,
    catalogState,
    setCatalogState,
    selectionState,
    setSelectionState,
    setSelectedWorkflow,
    setWorkflowWorkspaceMode,
    setWorkflowQuery,
    setErrorMessage,
    setStatusMessage,
    setCatalogQuery,
    setCatalogCategory,
    setCatalogInsertMode,
    setInspectorNode,
    setSelectedStepId,
  };
}

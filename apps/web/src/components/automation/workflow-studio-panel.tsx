"use client";

import { useCallback, type FormEvent, useEffect, useMemo, useRef, useState } from "react";
import { useRouter } from "next/navigation";

import type { WorkflowResponse, WorkflowRunResponse, WorkflowStepDto } from "@/lib/api";
import { WorkflowFlowView } from "@/components/automation/workflow-studio/flow-view/workflow-flow-view";
import { useRuntimeSchemas } from "@/components/automation/workflow-studio/hooks/use-runtime-schemas";
import {
  useWorkflowStudioState,
  type WorkflowWorkspaceMode,
} from "@/components/automation/workflow-studio/hooks/use-workflow-studio-state";
import { useWorkflowEditor } from "@/components/automation/workflow-studio/hooks/use-workflow-editor";
import { useWorkflowExecution } from "@/components/automation/workflow-studio/hooks/use-workflow-execution";
import {
  appendStepToBranch,
  buildStepPathIndex,
  cloneWorkflowSteps,
  collectWorkflowValidationIssues,
  createDraftFromTransport,
  createDraftStep,
  createTemplateStep,
  dynamicTokensForStep,
  duplicateStepById,
  findStepById,
  insertStepRelativeToTarget,
  isTypingElement,
  resolveTemplateList,
  stepTraceMapByPath,
  triggerTemplateConfig,
  updateStepById,
  type CanvasHistorySnapshot,
  type CatalogInsertMode,
  type DraftWorkflowStep,
  type FlowTemplateId,
  type TriggerType,
} from "@/components/automation/workflow-studio/model";
import { NodePickerDialog } from "@/components/automation/workflow-studio/panels/node-picker-dialog";
import { WorkflowBuilderPanel } from "@/components/automation/workflow-studio/panels/workflow-builder-panel";
import { WorkflowStudioToolbar } from "@/components/automation/workflow-studio/panels/workflow-studio-toolbar";

type WorkflowStudioPanelProps = {
  workflows: WorkflowResponse[];
  runs: WorkflowRunResponse[];
  initialSelectedWorkflow?: string;
  initialWorkspaceMode?: WorkflowWorkspaceMode;
};

export type { WorkflowWorkspaceMode };

export function WorkflowStudioPanel(props: WorkflowStudioPanelProps) {
  return useWorkflowStudioPanelContent(props);
}

function useWorkflowStudioPanelContent({
  workflows,
  runs,
  initialSelectedWorkflow,
  initialWorkspaceMode,
}: WorkflowStudioPanelProps) {
  const router = useRouter();
  const {
    createId,
    workspaceState,
    catalogState,
    selectionState,
    setSelectedWorkflow,
    setWorkflowWorkspaceMode,
    setErrorMessage,
    setStatusMessage,
    setCatalogQuery,
    setCatalogCategory,
    setCatalogInsertMode,
    setInspectorNode,
    setSelectedStepId,
  } = useWorkflowStudioState({
    workflows,
    initialSelectedWorkflow,
    initialWorkspaceMode,
  });

  const selectedWorkflow = workspaceState.selectedWorkflow;
  const errorMessage = workspaceState.errorMessage;
  const statusMessage = workspaceState.statusMessage;
  const selectedStepId = selectionState.selectedStepId;
  const inspectorNode = selectionState.inspectorNode;
  const catalogQuery = catalogState.catalogQuery;
  const catalogCategory = catalogState.catalogCategory;
  const catalogInsertMode = catalogState.catalogInsertMode;

  const [showBuilderPanel, setShowBuilderPanel] = useState(true);
  const [showNodePicker, setShowNodePicker] = useState(false);
  const [nodePickerQuery, setNodePickerQuery] = useState("");
  const [nodePickerCategory, setNodePickerCategory] = useState<
    "all" | (typeof catalogCategory)
  >("all");
  const [nodePickerInsertMode, setNodePickerInsertMode] =
    useState<CatalogInsertMode>("after_selected");
  const [undoStack, setUndoStack] = useState<CanvasHistorySnapshot[]>([]);
  const [redoStack, setRedoStack] = useState<CanvasHistorySnapshot[]>([]);
  const [expandedNodeId, setExpandedNodeId] = useState<string | null>(null);
  const suppressHistoryRef = useRef(false);
  const initializedFromRouteRef = useRef(false);
  const nodePickerInputRef = useRef<HTMLInputElement | null>(null);

  const [steps, setSteps] = useState<DraftWorkflowStep[]>([
    createDraftStep("log_message", createId),
  ]);

  const {
    logicalName,
    setLogicalName,
    displayName,
    setDisplayName,
    description,
    setDescription,
    triggerType,
    setTriggerType,
    triggerEntityLogicalName,
    setTriggerEntityLogicalName,
    maxAttempts,
    setMaxAttempts,
    isEnabled,
    setIsEnabled,
    isSaving,
    saveWorkflow,
  } = useWorkflowEditor({
    onResetMessages: resetMessages,
    onStatusMessage: setStatusMessage,
    onErrorMessage: setErrorMessage,
    onRefresh: () => router.refresh(),
  });

  const {
    runtimeEntities,
    publishedSchemasByEntity,
  } = useRuntimeSchemas(triggerType, triggerEntityLogicalName);

  const {
    executePayload,
    setExecutePayload,
    attemptsByRun,
    expandedRunId,
    isExecuting,
    isRetryingStep,
    handleExecuteWorkflow,
    retryRunStep,
  } = useWorkflowExecution({
    selectedWorkflow,
    onResetMessages: resetMessages,
    onStatusMessage: setStatusMessage,
    onErrorMessage: setErrorMessage,
    onRefresh: () => router.refresh(),
  });

  const selectedStep = useMemo(
    () => (selectedStepId ? findStepById(steps, selectedStepId) : null),
    [selectedStepId, steps],
  );

  const filteredTemplates = useMemo(
    () => resolveTemplateList(catalogQuery.trim().toLowerCase(), catalogCategory),
    [catalogCategory, catalogQuery],
  );
  const nodePickerTemplates = useMemo(
    () => resolveTemplateList(nodePickerQuery.trim().toLowerCase(), nodePickerCategory),
    [nodePickerCategory, nodePickerQuery],
  );

  const canInsertIntoConditionBranch = selectedStep?.type === "condition";
  const validationIssues = useMemo(
    () => collectWorkflowValidationIssues(triggerType, triggerEntityLogicalName, steps),
    [steps, triggerEntityLogicalName, triggerType],
  );
  const validationErrorCount = validationIssues.filter((issue) => issue.level === "error").length;

  const runtimeEntityOptions = useMemo(
    () =>
      runtimeEntities
        .map((entity) => ({
          value: entity.logical_name,
          label: `${entity.display_name} (${entity.logical_name})`,
        }))
        .sort((left, right) => left.label.localeCompare(right.label)),
    [runtimeEntities],
  );

  const getEntityFieldPathSuggestions = useCallback(
    (entityLogicalName: string): string[] => {
      const normalized = entityLogicalName.trim();
      if (normalized.length === 0) {
        return [];
      }

      const schema = publishedSchemasByEntity[normalized];
      if (!schema) {
        return [];
      }

      return schema.fields
        .map((field) => field.logical_name)
        .sort((left, right) => left.localeCompare(right));
    },
    [publishedSchemasByEntity],
  );

  const triggerFieldPathSuggestions = useMemo(() => {
    if (triggerType === "manual" || triggerType === "schedule_tick") {
      return [];
    }

    return getEntityFieldPathSuggestions(triggerEntityLogicalName);
  }, [getEntityFieldPathSuggestions, triggerEntityLogicalName, triggerType]);

  const getAvailableTokensForStep = useCallback(
    (stepId: string) => dynamicTokensForStep(steps, stepId, triggerFieldPathSuggestions),
    [steps, triggerFieldPathSuggestions],
  );

  const stepPathIndex = useMemo(() => buildStepPathIndex(steps), [steps]);
  const activeRunAttempt = useMemo(() => {
    if (!expandedRunId) {
      return null;
    }

    const attempts = attemptsByRun[expandedRunId] ?? [];
    return attempts.at(-1) ?? null;
  }, [attemptsByRun, expandedRunId]);
  const activeStepTraceByPath = useMemo(
    () => stepTraceMapByPath(activeRunAttempt?.step_traces),
    [activeRunAttempt],
  );
  const activeRun = useMemo(() => {
    if (!expandedRunId) {
      return null;
    }

    return runs.find((run) => run.run_id === expandedRunId) ?? null;
  }, [expandedRunId, runs]);

  const leftPanelOffset = showBuilderPanel ? 300 : 0;

  function snapshotCanvasState(): CanvasHistorySnapshot {
    return {
      triggerType,
      triggerEntityLogicalName,
      steps: cloneWorkflowSteps(steps),
      selectedStepId,
      inspectorNode,
    };
  }

  function pushHistoryCheckpoint() {
    if (suppressHistoryRef.current) {
      return;
    }

    const snapshot = snapshotCanvasState();
    setUndoStack((current) => {
      const next = [...current, snapshot];
      return next.length > 100 ? next.slice(next.length - 100) : next;
    });
    setRedoStack([]);
  }

  function restoreSnapshot(snapshot: CanvasHistorySnapshot) {
    suppressHistoryRef.current = true;
    setTriggerType(snapshot.triggerType);
    setTriggerEntityLogicalName(snapshot.triggerEntityLogicalName);
    setSteps(cloneWorkflowSteps(snapshot.steps));
    setSelectedStepId(snapshot.selectedStepId);
    setInspectorNode(snapshot.inspectorNode);
    setTimeout(() => {
      suppressHistoryRef.current = false;
    }, 0);
  }

  function undoCanvasEdit() {
    setUndoStack((currentUndoStack) => {
      if (currentUndoStack.length === 0) {
        return currentUndoStack;
      }

      const nextUndoStack = currentUndoStack.slice(0, -1);
      const previousSnapshot = currentUndoStack[currentUndoStack.length - 1];
      setRedoStack((currentRedoStack) => [...currentRedoStack, snapshotCanvasState()]);
      restoreSnapshot(previousSnapshot);
      setStatusMessage("Undid last canvas edit.");
      return nextUndoStack;
    });
  }

  function redoCanvasEdit() {
    setRedoStack((currentRedoStack) => {
      if (currentRedoStack.length === 0) {
        return currentRedoStack;
      }

      const nextRedoStack = currentRedoStack.slice(0, -1);
      const nextSnapshot = currentRedoStack[currentRedoStack.length - 1];
      setUndoStack((currentUndoStack) => [...currentUndoStack, snapshotCanvasState()]);
      restoreSnapshot(nextSnapshot);
      setStatusMessage("Redid canvas edit.");
      return nextRedoStack;
    });
  }

  function resetMessages() {
    setErrorMessage(null);
    setStatusMessage(null);
  }

  function workflowModePath(workflowLogicalName: string, mode: WorkflowWorkspaceMode): string {
    return `/maker/automation/${encodeURIComponent(workflowLogicalName)}/${mode}`;
  }

  function openNodePicker() {
    setNodePickerQuery("");
    setNodePickerCategory("all");
    setNodePickerInsertMode(catalogInsertMode);
    setShowNodePicker(true);
  }

  function openNodePickerForInsert(mode: CatalogInsertMode, stepId?: string) {
    if (stepId) {
      selectStep(stepId);
    }

    setNodePickerQuery("");
    setNodePickerCategory("all");
    setNodePickerInsertMode(mode);
    setShowNodePicker(true);
  }

  function closeNodePicker() {
    setShowNodePicker(false);
  }

  function insertFromNodePicker(templateId: FlowTemplateId) {
    insertTemplateFromCatalog(templateId, {
      insertMode: nodePickerInsertMode,
    });
    setCatalogInsertMode(nodePickerInsertMode);
    setShowNodePicker(false);
  }

  function ensureAtLeastOneRootStep() {
    setSteps((current) => {
      if (current.length > 0) {
        return current;
      }

      const defaultStep = createDraftStep("log_message", createId);
      setSelectedStepId(defaultStep.id);
      return [defaultStep];
    });
  }

  function selectStep(stepId: string) {
    setSelectedStepId(stepId);
    setInspectorNode("step");
  }

  function updateTriggerType(nextTriggerType: TriggerType) {
    pushHistoryCheckpoint();
    setTriggerType(nextTriggerType);
  }

  function updateTriggerEntity(nextTriggerEntityLogicalName: string) {
    pushHistoryCheckpoint();
    setTriggerEntityLogicalName(nextTriggerEntityLogicalName);
  }

  function applyTriggerTemplate(templateId: FlowTemplateId) {
    pushHistoryCheckpoint();

    const config = triggerTemplateConfig(templateId);
    if (!config) {
      return;
    }

    setTriggerType(config.triggerType);
    setTriggerEntityLogicalName(config.triggerEntityLogicalName);
    setInspectorNode("trigger");
    setSelectedStepId(null);
    setStatusMessage(`Trigger updated to ${config.statusLabel}.`);
  }

  function addRootStep(stepType: DraftWorkflowStep["type"]) {
    pushHistoryCheckpoint();
    const draftStep = createDraftStep(stepType, createId);
    setSteps((current) => [...current, draftStep]);
    selectStep(draftStep.id);
  }

  function insertTemplateFromCatalog(
    templateId: FlowTemplateId,
    options?: {
      insertMode?: CatalogInsertMode;
    },
  ) {
    const template = filteredTemplates.find((entry) => entry.id === templateId)
      ?? nodePickerTemplates.find((entry) => entry.id === templateId)
      ?? resolveTemplateList("", "all").find((entry) => entry.id === templateId);
    if (!template) {
      return;
    }

    if (template.target === "trigger") {
      applyTriggerTemplate(templateId);
      return;
    }

    pushHistoryCheckpoint();
    const draftStep = createTemplateStep(templateId, createId);
    const selectedId = selectedStepId;
    const insertMode = options?.insertMode ?? catalogInsertMode;

    setSteps((current) => {
      if (insertMode === "root" || !selectedId) {
        return [...current, draftStep];
      }

      if (insertMode === "after_selected") {
        const insertion = insertStepRelativeToTarget(current, selectedId, "after", draftStep);
        return insertion.inserted ? insertion.steps : [...current, draftStep];
      }

      const selectedInCurrent = findStepById(current, selectedId);
      if (selectedInCurrent?.type !== "condition") {
        return [...current, draftStep];
      }

      return appendStepToBranch(
        current,
        selectedId,
        insertMode === "then_selected" ? "then" : "else",
        draftStep,
      );
    });

    selectStep(draftStep.id);
    setStatusMessage("Function added to canvas.");
  }

  function handleExpandNode(nodeId: string | null) {
    setExpandedNodeId(nodeId);
    if (nodeId && nodeId !== "trigger") {
      setSelectedStepId(nodeId);
      setInspectorNode("step");
    } else {
      setSelectedStepId(null);
      setInspectorNode("trigger");
    }
  }

  function handleUpdateStepById(
    stepId: string,
    updater: (step: DraftWorkflowStep) => DraftWorkflowStep,
  ) {
    pushHistoryCheckpoint();
    setSteps((current) => updateStepById(current, stepId, updater));
  }

  function handleRemoveStepById(stepId: string) {
    pushHistoryCheckpoint();
    if (expandedNodeId === stepId) {
      setExpandedNodeId(null);
      setSelectedStepId(null);
      setInspectorNode("trigger");
    }
    setSteps((current) => current.filter((step) => step.id !== stepId));
    setTimeout(() => {
      ensureAtLeastOneRootStep();
    }, 0);
  }

  function handleDuplicateStepById(stepId: string) {
    pushHistoryCheckpoint();
    setSteps((current) => {
      const result = duplicateStepById(current, stepId, createId);
      if (!result.duplicatedStepId) {
        setErrorMessage("Unable to duplicate step.");
        return current;
      }
      setExpandedNodeId(result.duplicatedStepId);
      setSelectedStepId(result.duplicatedStepId);
      setInspectorNode("step");
      setStatusMessage("Step duplicated.");
      setErrorMessage(null);
      return result.steps;
    });
  }

  function focusValidationIssue(issue: { stepId: string | null }) {
    handleExpandNode(issue.stepId ?? "trigger");
  }

  function loadWorkflowIntoBuilder(
    workflow: WorkflowResponse,
    options?: { pushRoute?: boolean },
  ) {
    resetMessages();
    setSelectedWorkflow(workflow.logical_name);
    setWorkflowWorkspaceMode("edit");
    if (options?.pushRoute !== false) {
      router.push(workflowModePath(workflow.logical_name, "edit"));
    }

    setUndoStack([]);
    setRedoStack([]);

    setLogicalName(workflow.logical_name);
    setDisplayName(workflow.display_name);
    setDescription(workflow.description ?? "");
    setTriggerType(workflow.trigger_type as TriggerType);
    setTriggerEntityLogicalName(workflow.trigger_entity_logical_name ?? "");
    setMaxAttempts(String(workflow.max_attempts));
    setIsEnabled(workflow.is_enabled);

    const transportSteps = Array.isArray(workflow.steps)
      ? workflow.steps
      : [
          {
            type: workflow.action_type,
            entity_logical_name: workflow.action_entity_logical_name,
            message: (workflow.action_payload as { message?: string }).message,
            data: workflow.action_payload,
          } as unknown as WorkflowStepDto,
        ];

    const draftSteps = transportSteps.map((step) => createDraftFromTransport(step, createId));
    setSteps(draftSteps);

    const firstStepId = draftSteps.at(0)?.id ?? null;
    setSelectedStepId(firstStepId);
    setInspectorNode(firstStepId ? "step" : "trigger");
    setStatusMessage(`Loaded ${workflow.display_name} into the flow canvas.`);
  }

  function openWorkflowHistory(
    workflow: WorkflowResponse,
    options?: { pushRoute?: boolean },
  ) {
    resetMessages();
    setSelectedWorkflow(workflow.logical_name);
    setWorkflowWorkspaceMode("history");
    if (options?.pushRoute !== false) {
      router.push(workflowModePath(workflow.logical_name, "history"));
    }
  }

  async function handleSaveWorkflow(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (validationErrorCount > 0) {
      setErrorMessage("Resolve flow checker errors before saving.");
      return;
    }
    await saveWorkflow(steps);
  }

  function handleExecutionWorkflowChange(nextWorkflow: string) {
    setSelectedWorkflow(nextWorkflow);
    if (!nextWorkflow) {
      router.push("/maker/automation");
      return;
    }

    router.push(`/maker/automation/${encodeURIComponent(nextWorkflow)}/edit`);
  }

  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      const key = event.key.toLowerCase();

      if (showNodePicker) {
        if (event.metaKey || event.ctrlKey) {
          const isUndo = key === "z" && !event.shiftKey;
          const isRedo = (key === "z" && event.shiftKey) || key === "y";
          if (isUndo) {
            event.preventDefault();
            undoCanvasEdit();
          } else if (isRedo) {
            event.preventDefault();
            redoCanvasEdit();
          }
        }
        return;
      }

      if (event.metaKey || event.ctrlKey) {
        const isUndo = key === "z" && !event.shiftKey;
        const isRedo = (key === "z" && event.shiftKey) || key === "y";
        if (isUndo) {
          event.preventDefault();
          undoCanvasEdit();
        } else if (isRedo) {
          event.preventDefault();
          redoCanvasEdit();
        }
        return;
      }

      if (key === "a" && !event.altKey && !event.shiftKey && !isTypingElement(event.target)) {
        event.preventDefault();
        openNodePicker();
      }
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  // eslint-disable-next-line react-hooks/exhaustive-deps -- Keyboard handler intentionally keyed to picker visibility.
  }, [showNodePicker]);

  useEffect(() => {
    if (!showNodePicker) {
      return;
    }

    const timeoutId = window.setTimeout(() => {
      nodePickerInputRef.current?.focus();
    }, 0);

    return () => {
      window.clearTimeout(timeoutId);
    };
  }, [showNodePicker]);

  useEffect(() => {
    if (initializedFromRouteRef.current) {
      return;
    }

    if (!initialSelectedWorkflow) {
      initializedFromRouteRef.current = true;
      return;
    }

    const workflow = workflows.find((entry) => entry.logical_name === initialSelectedWorkflow);
    if (!workflow) {
      initializedFromRouteRef.current = true;
      return;
    }

    const timeoutId = window.setTimeout(() => {
      if (initialWorkspaceMode === "history") {
        openWorkflowHistory(workflow, { pushRoute: false });
      } else {
        loadWorkflowIntoBuilder(workflow, { pushRoute: false });
      }
    }, 0);

    initializedFromRouteRef.current = true;
    return () => {
      window.clearTimeout(timeoutId);
    };
  // eslint-disable-next-line react-hooks/exhaustive-deps -- Route initialization should run once per mount.
  }, [initialSelectedWorkflow, initialWorkspaceMode, workflows]);

  return (
    <div className="relative h-[calc(100vh-9rem)] min-h-[760px] overflow-hidden rounded-xl border border-zinc-200 bg-zinc-50">
      <WorkflowStudioToolbar
        selectedWorkflow={selectedWorkflow}
        validationErrorCount={validationErrorCount}
        errorMessage={errorMessage}
        statusMessage={statusMessage}
        undoDisabled={undoStack.length === 0}
        redoDisabled={redoStack.length === 0}
        showBuilderPanel={showBuilderPanel}
        onUndo={undoCanvasEdit}
        onRedo={redoCanvasEdit}
        onOpenNodePicker={openNodePicker}
        onToggleBuilderPanel={() => setShowBuilderPanel((current) => !current)}
      />

      <NodePickerDialog
        open={showNodePicker}
        inputRef={nodePickerInputRef}
        query={nodePickerQuery}
        category={nodePickerCategory}
        insertMode={nodePickerInsertMode}
        canInsertIntoConditionBranch={canInsertIntoConditionBranch}
        templates={nodePickerTemplates}
        onQueryChange={setNodePickerQuery}
        onCategoryChange={setNodePickerCategory}
        onInsertModeChange={setNodePickerInsertMode}
        onInsert={insertFromNodePicker}
        onClose={closeNodePicker}
      />

      <WorkflowBuilderPanel
        open={showBuilderPanel}
        onSaveWorkflow={handleSaveWorkflow}
        logicalName={logicalName}
        onLogicalNameChange={setLogicalName}
        displayName={displayName}
        onDisplayNameChange={setDisplayName}
        description={description}
        onDescriptionChange={setDescription}
        maxAttempts={maxAttempts}
        onMaxAttemptsChange={setMaxAttempts}
        isEnabled={isEnabled}
        onEnabledChange={setIsEnabled}
        catalogQuery={catalogQuery}
        onCatalogQueryChange={setCatalogQuery}
        catalogCategory={catalogCategory}
        onCatalogCategoryChange={setCatalogCategory}
        catalogInsertMode={catalogInsertMode}
        canInsertIntoConditionBranch={canInsertIntoConditionBranch}
        filteredTemplates={filteredTemplates}
        onInsertTemplate={insertTemplateFromCatalog}
        onAddRootStep={addRootStep}
        isSaving={isSaving}
        onExecuteWorkflow={handleExecuteWorkflow}
        onExecutionWorkflowChange={handleExecutionWorkflowChange}
        workflows={workflows}
        selectedWorkflow={selectedWorkflow}
        executePayload={executePayload}
        onExecutePayloadChange={setExecutePayload}
        isExecuting={isExecuting}
        validationIssues={validationIssues}
        validationErrorCount={validationErrorCount}
        onFocusValidationIssue={focusValidationIssue}
      />

      <div
        className="absolute inset-0 transition-[padding]"
        style={{ paddingTop: "48px", paddingLeft: `${leftPanelOffset}px` }}
      >
        <WorkflowFlowView
          steps={steps}
          triggerType={triggerType}
          triggerEntityLogicalName={triggerEntityLogicalName}
          expandedNodeId={expandedNodeId}
          onExpandNode={handleExpandNode}
          onUpdateStep={handleUpdateStepById}
          onRemoveStep={handleRemoveStepById}
          onDuplicateStep={handleDuplicateStepById}
          onOpenNodePicker={openNodePickerForInsert}
          getAvailableTokens={getAvailableTokensForStep}
          runtimeEntityOptions={runtimeEntityOptions}
          triggerFieldPathSuggestions={triggerFieldPathSuggestions}
          getEntityFieldPathSuggestions={getEntityFieldPathSuggestions}
          onTriggerTypeChange={updateTriggerType}
          onTriggerEntityChange={updateTriggerEntity}
          stepTraceByPath={activeStepTraceByPath}
          stepPathByStepId={stepPathIndex.byStepId}
          isRetryingStep={isRetryingStep}
          onRetryStep={(stepPath, strategy, backoffMs) => {
            if (!activeRun) {
              setErrorMessage("Select a run from history before retrying a failed step.");
              return;
            }
            void retryRunStep(
              activeRun.workflow_logical_name,
              activeRun.run_id,
              stepPath,
              strategy,
              backoffMs,
            );
          }}
        />
      </div>
    </div>
  );
}

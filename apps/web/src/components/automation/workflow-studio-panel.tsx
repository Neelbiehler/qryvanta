"use client";

import {
  useCallback,
  type FormEvent,
  type PointerEvent as ReactPointerEvent,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { useRouter } from "next/navigation";

import {
  Button,
  Input,
  Label,
  Select,
  Separator,
  StatusBadge,
  Textarea,
} from "@qryvanta/ui";

import {
  apiFetch,
  type WorkflowConditionOperatorDto,
  type WorkflowResponse,
  type WorkflowRunResponse,
  type WorkflowStepDto,
} from "@/lib/api";
import { NodePickerDialog } from "@/components/automation/workflow-studio/panels/node-picker-dialog";
import { WorkflowBuilderPanel } from "@/components/automation/workflow-studio/panels/workflow-builder-panel";
import { WorkflowInspectorPanel } from "@/components/automation/workflow-studio/panels/workflow-inspector-panel";
import { WorkflowCanvasScene } from "@/components/automation/workflow-studio/canvas/workflow-canvas-scene";
import { useCanvasState } from "@/components/automation/workflow-studio/hooks/use-canvas-state";
import { useWorkflowEditor } from "@/components/automation/workflow-studio/hooks/use-workflow-editor";
import { useWorkflowExecution } from "@/components/automation/workflow-studio/hooks/use-workflow-execution";
import {
  CANVAS_NODE_HEIGHT,
  CANVAS_NODE_WIDTH,
  CANVAS_PADDING,
  CONDITION_OPERATORS,
  FLOW_TEMPLATES,
  GRID_SIZE,
  LANE_WIDTH,
  STEP_LIBRARY,
  TRIGGER_NODE_ID,
  appendStepToBranch,
  buildCanvasGraph,
  buildDefaultCanvasPositions,
  cloneCanvasPositions,
  cloneWorkflowSteps,
  createDraftFromTransport,
  createDraftStep,
  createTemplateStep,
  describeTrigger,
  extractStepById,
  findStepById,
  insertStepRelativeToTarget,
  isTypingElement,
  maxCanvasDepth,
  removeStepById,
  rerouteTargetFromDataset,
  rerouteTargetsEqual,
  resolveTemplateList,
  stepContainsId,
  updateStepById,
  type CanvasHistorySnapshot,
  type CanvasPosition,
  type CatalogInsertMode,
  type DraftWorkflowStep,
  type FlowTemplateCategory,
  type FlowTemplateId,
  type InspectorNode,
  type RerouteTarget,
  type SelectionBoxState,
  type TriggerType,
} from "@/components/automation/workflow-studio/model";

type WorkflowStudioPanelProps = {
  workflows: WorkflowResponse[];
  runs: WorkflowRunResponse[];
  initialSelectedWorkflow?: string;
  initialWorkspaceMode?: WorkflowWorkspaceMode;
};

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
  const idCounterRef = useRef(1);

  function createId(): string {
    const id = `flow_step_${idCounterRef.current}`;
    idCounterRef.current += 1;
    return id;
  }


  const [steps, setSteps] = useState<DraftWorkflowStep[]>([
    createDraftStep("log_message", createId),
  ]);

  const [selectionState, setSelectionState] = useState<{
    inspectorNode: InspectorNode;
    selectedStepId: string | null;
  }>({
    inspectorNode: "trigger",
    selectedStepId: null,
  });

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

  const selectedWorkflow = workspaceState.selectedWorkflow;
  const workflowWorkspaceMode = workspaceState.workflowWorkspaceMode;
  const workflowQuery = workspaceState.workflowQuery;
  const errorMessage = workspaceState.errorMessage;
  const statusMessage = workspaceState.statusMessage;
  const inspectorNode = selectionState.inspectorNode;
  const selectedStepId = selectionState.selectedStepId;
  const catalogQuery = catalogState.catalogQuery;
  const catalogCategory = catalogState.catalogCategory;
  const catalogInsertMode = catalogState.catalogInsertMode;

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

  const setCatalogCategory = useCallback(
    (next: "all" | FlowTemplateCategory) => {
      setCatalogState((current) => ({ ...current, catalogCategory: next }));
    },
    [],
  );

  const setCatalogInsertMode = useCallback((next: CatalogInsertMode) => {
    setCatalogState((current) => ({ ...current, catalogInsertMode: next }));
  }, []);

  const setInspectorNode = useCallback((next: InspectorNode) => {
    setSelectionState((current) => ({ ...current, inspectorNode: next }));
  }, []);

  const setSelectedStepId = useCallback((next: string | null) => {
    setSelectionState((current) => ({ ...current, selectedStepId: next }));
  }, []);

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
    executePayload,
    setExecutePayload,
    attemptsByRun,
    expandedRunId,
    isExecuting,
    handleExecuteWorkflow,
    toggleAttempts,
  } = useWorkflowExecution({
    selectedWorkflow,
    onResetMessages: resetMessages,
    onStatusMessage: setStatusMessage,
    onErrorMessage: setErrorMessage,
    onRefresh: () => router.refresh(),
  });

  const {
    showBuilderPanel,
    setShowBuilderPanel,
    showInspectorPanel,
    setShowInspectorPanel,
    showNodePicker,
    setShowNodePicker,
    nodePickerQuery,
    setNodePickerQuery,
    nodePickerCategory,
    setNodePickerCategory,
    nodePickerInsertMode,
    setNodePickerInsertMode,
    snapToGrid,
    setSnapToGrid,
    wiringSourceStepId,
    setWiringSourceStepId,
    selectedCanvasNodeIds,
    setSelectedCanvasNodeIds,
    selectionBox,
    setSelectionBox,
    connectionDrag,
    setConnectionDrag,
    nodePositions,
    setNodePositions,
    undoStack,
    setUndoStack,
    redoStack,
    setRedoStack,
    dragState,
    setDragState,
    canvasRef,
    nodePickerInputRef,
    lastCanvasPointerRef,
    suppressHistoryRef,
    initializedFromRouteRef,
  } = useCanvasState();

  const selectedStep = useMemo(
    () => (selectedStepId ? findStepById(steps, selectedStepId) : null),
    [selectedStepId, steps],
  );
  const wiringSourceStep = useMemo(
    () => (wiringSourceStepId ? findStepById(steps, wiringSourceStepId) : null),
    [wiringSourceStepId, steps],
  );
  const normalizedCatalogQuery = catalogQuery.trim().toLowerCase();
  const normalizedNodePickerQuery = nodePickerQuery.trim().toLowerCase();
  const normalizedWorkflowQuery = workflowQuery.trim().toLowerCase();
  const filteredWorkflows = useMemo(
    () =>
      workflows.filter((workflow) => {
        if (normalizedWorkflowQuery.length === 0) {
          return true;
        }

        const searchable = `${workflow.display_name} ${workflow.logical_name}`.toLowerCase();
        return searchable.includes(normalizedWorkflowQuery);
      }),
    [normalizedWorkflowQuery, workflows],
  );
  const selectedWorkflowRuns = useMemo(
    () =>
      selectedWorkflow
        ? runs.filter((run) => run.workflow_logical_name === selectedWorkflow)
        : runs,
    [runs, selectedWorkflow],
  );

  const filteredTemplates = useMemo(() => {
    return resolveTemplateList(normalizedCatalogQuery, catalogCategory);
  }, [catalogCategory, normalizedCatalogQuery]);
  const nodePickerTemplates = useMemo(() => {
    return resolveTemplateList(normalizedNodePickerQuery, nodePickerCategory);
  }, [nodePickerCategory, normalizedNodePickerQuery]);
  const canInsertIntoConditionBranch = selectedStep?.type === "condition";

  const triggerSummary = describeTrigger(triggerType, triggerEntityLogicalName);
  const canvasGraph = useMemo(
    () => buildCanvasGraph(triggerSummary, steps),
    [triggerSummary, steps],
  );
  const laneCount = useMemo(() => Math.max(3, maxCanvasDepth(steps) + 1), [steps]);
  const canvasSurfaceWidth = useMemo(() => {
    const maxPositionX = Math.max(
      0,
      ...Object.values(nodePositions).map((position) => position.x),
    );
    return Math.max(1320, 320 + laneCount * LANE_WIDTH, maxPositionX + CANVAS_NODE_WIDTH + 260);
  }, [laneCount, nodePositions]);
  const canvasSurfaceHeight = useMemo(() => {
    const maxPositionY = Math.max(
      0,
      ...Object.values(nodePositions).map((position) => position.y),
    );
    return Math.max(760, maxPositionY + CANVAS_NODE_HEIGHT + 220);
  }, [nodePositions]);
  const leftPanelOffset = showBuilderPanel ? 352 : 24;
  const rightPanelOffset = showInspectorPanel ? 372 : 24;

  const pointerToCanvasPosition = useCallback(
    (clientX: number, clientY: number): CanvasPosition | null => {
      const canvasElement = canvasRef.current;
      if (!canvasElement) {
        return null;
      }

      const rect = canvasElement.getBoundingClientRect();
      return {
        x: clientX - rect.left + canvasElement.scrollLeft,
        y: clientY - rect.top + canvasElement.scrollTop,
      };
    },
    [canvasRef],
  );

  function targetFromPointer(clientX: number, clientY: number): RerouteTarget | null {
    const targetElement = (document.elementFromPoint(clientX, clientY) as HTMLElement | null)
      ?.closest("[data-wire-target-kind]") as HTMLElement | null;

    if (!targetElement) {
      return null;
    }

    return rerouteTargetFromDataset(
      targetElement.dataset.wireTargetKind,
      targetElement.dataset.wireTargetId,
    );
  }

  function snapshotCanvasState(): CanvasHistorySnapshot {
    return {
      triggerType,
      triggerEntityLogicalName,
      steps: cloneWorkflowSteps(steps),
      nodePositions: cloneCanvasPositions(nodePositions),
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
    setNodePositions(cloneCanvasPositions(snapshot.nodePositions));
    setSelectedStepId(snapshot.selectedStepId);
    setSelectedCanvasNodeIds(snapshot.selectedStepId ? [snapshot.selectedStepId] : []);
    setInspectorNode(snapshot.inspectorNode);
    setWiringSourceStepId(null);
    setConnectionDrag(null);
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

  useEffect(() => {
    if (wiringSourceStepId && !wiringSourceStep) {
      setConnectionDrag(null);
      setWiringSourceStepId(null);
    }
  }, [setConnectionDrag, setWiringSourceStepId, wiringSourceStep, wiringSourceStepId]);

  useEffect(() => {
    const defaults = buildDefaultCanvasPositions(steps);
    const validNodeIds = new Set(canvasGraph.nodes.map((node) => node.id));

    setNodePositions((current) => {
      let changed = false;
      const next: Record<string, CanvasPosition> = {};

      for (const node of canvasGraph.nodes) {
        if (current[node.id]) {
          next[node.id] = current[node.id];
        } else {
          next[node.id] = defaults[node.id] ?? { x: CANVAS_PADDING, y: CANVAS_PADDING };
          changed = true;
        }
      }

      for (const existingId of Object.keys(current)) {
        if (!validNodeIds.has(existingId)) {
          changed = true;
          break;
        }
      }

      if (!changed && Object.keys(current).length === Object.keys(next).length) {
        return current;
      }

      return next;
    });
  }, [canvasGraph.nodes, setNodePositions, steps]);

  function markCanvasDragMoved() {
    pushHistoryCheckpoint();
    setDragState((current) =>
      current ? { ...current, hasMoved: true } : current,
    );
  }

  function applyCanvasDragPositions(
    activeDrag: NonNullable<typeof dragState>,
    deltaX: number,
    deltaY: number,
    maxX: number,
    maxY: number,
  ) {
    setNodePositions((current) => {
      const next = { ...current };

      for (const dragNodeId of activeDrag.nodeIds) {
        const initialPosition = activeDrag.initialPositions[dragNodeId] ?? {
          x: CANVAS_PADDING,
          y: CANVAS_PADDING,
        };

        let x = Math.min(maxX, Math.max(CANVAS_PADDING, initialPosition.x + deltaX));
        let y = Math.min(maxY, Math.max(CANVAS_PADDING, initialPosition.y + deltaY));

        if (snapToGrid) {
          x = Math.round(x / GRID_SIZE) * GRID_SIZE;
          y = Math.round(y / GRID_SIZE) * GRID_SIZE;
        }

        next[dragNodeId] = { x, y };
      }

      return next;
    });
  }

  function endCanvasDrag() {
    setDragState(null);
  }

  function updateConnectionDragPointer(
    pointerPosition: CanvasPosition,
    hoveredTarget: RerouteTarget | null,
  ) {
    setConnectionDrag((current) =>
      current
        ? {
            ...current,
            pointerX: pointerPosition.x,
            pointerY: pointerPosition.y,
            hoveredTarget,
          }
        : current,
    );
  }

  function completeConnectionDrag(
    event: PointerEvent,
    activeConnectionDrag: NonNullable<typeof connectionDrag>,
    activeSourceStepId: string,
  ) {
    const hoveredTarget =
      targetFromPointer(event.clientX, event.clientY) ??
      activeConnectionDrag.hoveredTarget;

    setConnectionDrag(null);

    if (hoveredTarget) {
      rerouteStep(hoveredTarget, activeSourceStepId);
      return;
    }

    setStatusMessage("Connection cancelled.");
  }

  const updateSelectionBoxPointer = useCallback(
    (pointerPosition: CanvasPosition) => {
      setSelectionBox((current) =>
        current
          ? {
              ...current,
              currentX: pointerPosition.x,
              currentY: pointerPosition.y,
            }
          : current,
      );
    },
    [setSelectionBox],
  );

  const completeSelectionBoxSelection = useCallback(() => {
    setSelectionBox((current) => {
      if (!current) {
        return null;
      }

      const left = Math.min(current.startX, current.currentX);
      const right = Math.max(current.startX, current.currentX);
      const top = Math.min(current.startY, current.currentY);
      const bottom = Math.max(current.startY, current.currentY);

      const selectedIds = canvasGraph.nodes
        .filter((node) => {
          const position = nodePositions[node.id];
          if (!position) {
            return false;
          }

          const nodeLeft = position.x;
          const nodeRight = position.x + CANVAS_NODE_WIDTH;
          const nodeTop = position.y;
          const nodeBottom = position.y + CANVAS_NODE_HEIGHT;

          return !(
            nodeRight < left ||
            nodeLeft > right ||
            nodeBottom < top ||
            nodeTop > bottom
          );
        })
        .map((node) => node.id)
        .filter((nodeId) => nodeId !== TRIGGER_NODE_ID);

      setSelectedCanvasNodeIds(selectedIds);
      return null;
    });
  }, [canvasGraph.nodes, nodePositions, setSelectedCanvasNodeIds, setSelectionBox]);

  useEffect(() => {
    if (!dragState) {
      return;
    }

    const activeDrag = dragState;

    function handlePointerMove(event: PointerEvent) {
      const canvasElement = canvasRef.current;
      if (!canvasElement) {
        return;
      }

      const rect = canvasElement.getBoundingClientRect();
      const unclampedX =
        event.clientX - rect.left + canvasElement.scrollLeft;
      const unclampedY =
        event.clientY - rect.top + canvasElement.scrollTop;

      const deltaX = unclampedX - activeDrag.pointerStartX;
      const deltaY = unclampedY - activeDrag.pointerStartY;

      if (!activeDrag.hasMoved && (Math.abs(deltaX) > 1 || Math.abs(deltaY) > 1)) {
        markCanvasDragMoved();
      }

      const maxX = Math.max(
        CANVAS_PADDING,
        canvasElement.scrollWidth - CANVAS_NODE_WIDTH - CANVAS_PADDING,
      );
      const maxY = Math.max(
        CANVAS_PADDING,
        canvasElement.scrollHeight - CANVAS_NODE_HEIGHT - CANVAS_PADDING,
      );

      applyCanvasDragPositions(activeDrag, deltaX, deltaY, maxX, maxY);
    }

    function handlePointerUp() {
      endCanvasDrag();
    }

    window.addEventListener("pointermove", handlePointerMove);
    window.addEventListener("pointerup", handlePointerUp);

    return () => {
      window.removeEventListener("pointermove", handlePointerMove);
      window.removeEventListener("pointerup", handlePointerUp);
    };
  // eslint-disable-next-line react-hooks/exhaustive-deps -- Pointer drag listeners are intentionally keyed to drag state only.
  }, [dragState, snapToGrid]);

  useEffect(() => {
    if (!connectionDrag) {
      return;
    }

    const activeConnectionDrag = connectionDrag;
    const activeSourceStepId = connectionDrag.sourceStepId;

    function handlePointerMove(event: PointerEvent) {
      const pointerPosition = pointerToCanvasPosition(event.clientX, event.clientY);
      if (!pointerPosition) {
        return;
      }

      const hoveredTarget = targetFromPointer(event.clientX, event.clientY);
      updateConnectionDragPointer(pointerPosition, hoveredTarget);
    }

    function handlePointerUp(event: PointerEvent) {
      completeConnectionDrag(event, activeConnectionDrag, activeSourceStepId);
    }

    window.addEventListener("pointermove", handlePointerMove);
    window.addEventListener("pointerup", handlePointerUp);

    return () => {
      window.removeEventListener("pointermove", handlePointerMove);
      window.removeEventListener("pointerup", handlePointerUp);
    };
  // eslint-disable-next-line react-hooks/exhaustive-deps -- Connection drag listener is intentionally keyed to active drag session.
  }, [connectionDrag]);

  useEffect(() => {
    if (!selectionBox) {
      return;
    }

    function handlePointerMove(event: PointerEvent) {
      const pointerPosition = pointerToCanvasPosition(event.clientX, event.clientY);
      if (!pointerPosition) {
        return;
      }

      updateSelectionBoxPointer(pointerPosition);
    }

    function handlePointerUp() {
      completeSelectionBoxSelection();
    }

    window.addEventListener("pointermove", handlePointerMove);
    window.addEventListener("pointerup", handlePointerUp);

    return () => {
      window.removeEventListener("pointermove", handlePointerMove);
      window.removeEventListener("pointerup", handlePointerUp);
    };
  }, [
    canvasGraph.nodes,
    completeSelectionBoxSelection,
    nodePositions,
    pointerToCanvasPosition,
    selectionBox,
    setSelectedCanvasNodeIds,
    setSelectionBox,
    updateSelectionBoxPointer,
  ]);

  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      const key = event.key.toLowerCase();

      if (showNodePicker) {
        if (key === "escape") {
          event.preventDefault();
          closeNodePicker();
          return;
        }

        if (key === "enter") {
          const firstTemplate = nodePickerTemplates[0];
          if (firstTemplate) {
            event.preventDefault();
            insertFromNodePicker(firstTemplate.id);
          }
          return;
        }
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

      if (
        key === "a" &&
        !event.altKey &&
        !event.shiftKey &&
        !isTypingElement(event.target)
      ) {
        event.preventDefault();
        openNodePicker();
      }
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  // eslint-disable-next-line react-hooks/exhaustive-deps -- Keyboard handlers intentionally depend on picker visibility and template matches.
  }, [showNodePicker, nodePickerTemplates]);

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
  }, [nodePickerInputRef, showNodePicker]);

  useEffect(() => {
    if (initializedFromRouteRef.current) {
      return;
    }

    if (!initialSelectedWorkflow) {
      initializedFromRouteRef.current = true;
      return;
    }

    const workflow = workflows.find(
      (entry) => entry.logical_name === initialSelectedWorkflow,
    );
    if (!workflow) {
      initializedFromRouteRef.current = true;
      return;
    }

    if (initialWorkspaceMode === "history") {
      openWorkflowHistory(workflow, { pushRoute: false });
    } else {
      loadWorkflowIntoBuilder(workflow, { pushRoute: false });
    }

    initializedFromRouteRef.current = true;
  // eslint-disable-next-line react-hooks/exhaustive-deps -- Route-based initialization should run once per mount.
  }, [initialSelectedWorkflow, initialWorkspaceMode, workflows]);

  function resetMessages() {
    setErrorMessage(null);
    setStatusMessage(null);
  }

  function workflowModePath(
    workflowLogicalName: string,
    mode: WorkflowWorkspaceMode,
  ): string {
    return `/maker/automation/${encodeURIComponent(workflowLogicalName)}/${mode}`;
  }

  function navigateWorkspaceMode(mode: WorkflowWorkspaceMode) {
    setWorkflowWorkspaceMode(mode);

    if (selectedWorkflow) {
      router.push(workflowModePath(selectedWorkflow, mode));
      return;
    }

    if (mode === "history") {
      setErrorMessage("Select a workflow first.");
      return;
    }

    router.push("/maker/automation");
  }

  function openNodePicker() {
    setNodePickerQuery("");
    setNodePickerCategory("all");
    setNodePickerInsertMode(catalogInsertMode);
    setShowNodePicker(true);
  }

  function closeNodePicker() {
    setShowNodePicker(false);
  }

  function insertFromNodePicker(templateId: FlowTemplateId) {
    insertTemplateFromCatalog(templateId, {
      insertMode: nodePickerInsertMode,
      cursorPosition: lastCanvasPointerRef.current,
    });
    setCatalogInsertMode(nodePickerInsertMode);
    setShowNodePicker(false);
  }

  function resetCanvasLayout() {
    pushHistoryCheckpoint();
    setNodePositions(buildDefaultCanvasPositions(steps));
    setStatusMessage("Canvas auto-arranged.");
  }

  function cancelWireRouting() {
    setConnectionDrag(null);
    setWiringSourceStepId(null);
  }

  function startWireRouting(stepId: string) {
    setConnectionDrag(null);
    setWiringSourceStepId(stepId);
    setSelectedCanvasNodeIds([stepId]);
    setStatusMessage("Connection mode active. Choose a target handle to reroute this step.");
    setErrorMessage(null);
  }

  function beginWireDrag(stepId: string, event: ReactPointerEvent<HTMLButtonElement>) {
    event.preventDefault();
    event.stopPropagation();

    const pointerPosition = pointerToCanvasPosition(event.clientX, event.clientY);
    if (!pointerPosition) {
      return;
    }

    lastCanvasPointerRef.current = pointerPosition;

    setWiringSourceStepId(stepId);
    setSelectedCanvasNodeIds([stepId]);
    setErrorMessage(null);
    setStatusMessage("Drag to a target handle to reroute this step.");
    setConnectionDrag({
      sourceStepId: stepId,
      pointerX: pointerPosition.x,
      pointerY: pointerPosition.y,
      hoveredTarget: null,
    });
  }

  function isHoveredRerouteTarget(target: RerouteTarget): boolean {
    if (!connectionDrag?.hoveredTarget) {
      return false;
    }

    return rerouteTargetsEqual(connectionDrag.hoveredTarget, target);
  }

  function rerouteStep(target: RerouteTarget, sourceStepIdOverride?: string) {
    const sourceStepId = sourceStepIdOverride ?? wiringSourceStepId;
    if (!sourceStepId) {
      setErrorMessage("Select a source step first.");
      return;
    }

    const sourceStep = findStepById(steps, sourceStepId);
    if (!sourceStep) {
      setErrorMessage("Unable to resolve source step for reroute.");
      setWiringSourceStepId(null);
      return;
    }

    if (target.kind !== "trigger_start") {
      if (sourceStep.id === target.targetId) {
        setErrorMessage("Cannot wire a step to itself.");
        return;
      }

      if (stepContainsId(sourceStep, target.targetId)) {
        setErrorMessage("Cannot wire a step into its own nested branch.");
        return;
      }
    }

    const extraction = extractStepById(steps, sourceStep.id);
    if (!extraction.extracted) {
      setErrorMessage("Failed to detach source step for rerouting.");
      return;
    }

    let nextSteps: DraftWorkflowStep[];
    let inserted = true;

    if (target.kind === "trigger_start") {
      nextSteps = [extraction.extracted, ...extraction.steps];
    } else {
      const insertion = insertStepRelativeToTarget(
        extraction.steps,
        target.targetId,
        target.kind,
        extraction.extracted,
      );
      nextSteps = insertion.steps;
      inserted = insertion.inserted;
    }

    if (!inserted) {
      setErrorMessage("Unable to apply wire reroute target.");
      return;
    }

    pushHistoryCheckpoint();
    setSteps(nextSteps);
    setSelectedStepId(extraction.extracted.id);
    setInspectorNode("step");
    setWiringSourceStepId(null);
    setStatusMessage("Step routing updated.");
    setErrorMessage(null);
  }

  function beginNodeDrag(
    nodeId: string,
    event: ReactPointerEvent<HTMLButtonElement>,
    additiveSelection = false,
  ) {
    const canvasElement = canvasRef.current;
    if (!canvasElement) {
      return;
    }

    const pointerPosition = pointerToCanvasPosition(event.clientX, event.clientY);
    if (!pointerPosition) {
      return;
    }

    lastCanvasPointerRef.current = pointerPosition;

    let dragNodeIds: string[];
    if (additiveSelection) {
      setSelectedCanvasNodeIds((current) => {
        if (current.includes(nodeId)) {
          dragNodeIds = current;
          return current;
        }
        dragNodeIds = [...current, nodeId];
        return dragNodeIds;
      });
      dragNodeIds = selectedCanvasNodeIds.includes(nodeId)
        ? selectedCanvasNodeIds
        : [...selectedCanvasNodeIds, nodeId];
    } else {
      dragNodeIds = selectedCanvasNodeIds.includes(nodeId)
        ? selectedCanvasNodeIds
        : [nodeId];
      setSelectedCanvasNodeIds(dragNodeIds);
    }

    if (dragNodeIds.length === 0) {
      dragNodeIds = [nodeId];
    }

    const initialPositions: Record<string, CanvasPosition> = {};
    for (const dragNodeId of dragNodeIds) {
      initialPositions[dragNodeId] =
        nodePositions[dragNodeId] ?? ({ x: CANVAS_PADDING, y: CANVAS_PADDING } as CanvasPosition);
    }

    const rect = canvasElement.getBoundingClientRect();
    setDragState({
      nodeIds: dragNodeIds,
      pointerStartX: event.clientX - rect.left + canvasElement.scrollLeft,
      pointerStartY: event.clientY - rect.top + canvasElement.scrollTop,
      initialPositions,
      hasMoved: false,
    });
  }

  function beginSelectionBox(event: ReactPointerEvent<HTMLDivElement>) {
    if (dragState || connectionDrag) {
      return;
    }

    const canvasElement = canvasRef.current;
    if (!canvasElement) {
      return;
    }

    const target = event.target as HTMLElement;
    if (target.closest("[data-canvas-node='true']") || target.closest("[data-wire-target-kind]")) {
      return;
    }

    const pointerPosition = pointerToCanvasPosition(event.clientX, event.clientY);
    if (!pointerPosition) {
      return;
    }

    lastCanvasPointerRef.current = pointerPosition;

    if (!event.shiftKey) {
      setSelectedCanvasNodeIds([]);
    }

    setSelectionBox({
      startX: pointerPosition.x,
      startY: pointerPosition.y,
      currentX: pointerPosition.x,
      currentY: pointerPosition.y,
    });
  }

  function resetBuilder() {
    pushHistoryCheckpoint();
    setSelectedWorkflow("");
    setWorkflowWorkspaceMode("edit");
    setLogicalName("");
    setDisplayName("");
    setDescription("");
    setTriggerType("manual");
    setTriggerEntityLogicalName("");
    setMaxAttempts("3");
    setIsEnabled(true);
    const firstStep = createDraftStep("log_message", createId);
    setSteps([firstStep]);
    setSelectedStepId(firstStep.id);
    setSelectedCanvasNodeIds([firstStep.id]);
    setConnectionDrag(null);
    setWiringSourceStepId(null);
    setInspectorNode("trigger");
    resetMessages();
    router.push("/maker/automation");
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
    setSelectedCanvasNodeIds([stepId]);
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

    if (templateId === "webhook_trigger") {
      setTriggerType("runtime_record_created");
      setTriggerEntityLogicalName("webhook_event");
      setInspectorNode("trigger");
      setSelectedStepId(null);
      setStatusMessage("Trigger updated to Webhook Event.");
      return;
    }

    setTriggerType("manual");
    setTriggerEntityLogicalName("");
    setInspectorNode("trigger");
    setSelectedStepId(null);
    setStatusMessage("Trigger updated to Manual.");
  }

  function placeNodeAtCursor(stepId: string, cursorPosition: CanvasPosition | null) {
    if (!cursorPosition) {
      return;
    }

    let x = cursorPosition.x - CANVAS_NODE_WIDTH / 2;
    let y = cursorPosition.y - CANVAS_NODE_HEIGHT / 2;

    if (snapToGrid) {
      x = Math.round(x / GRID_SIZE) * GRID_SIZE;
      y = Math.round(y / GRID_SIZE) * GRID_SIZE;
    }

    const maxX = Math.max(CANVAS_PADDING, canvasSurfaceWidth - CANVAS_NODE_WIDTH - CANVAS_PADDING);
    const maxY = Math.max(CANVAS_PADDING, canvasSurfaceHeight - CANVAS_NODE_HEIGHT - CANVAS_PADDING);
    x = Math.min(maxX, Math.max(CANVAS_PADDING, x));
    y = Math.min(maxY, Math.max(CANVAS_PADDING, y));

    setNodePositions((current) => ({
      ...current,
      [stepId]: { x, y },
    }));
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
      cursorPosition?: CanvasPosition | null;
    },
  ) {
    const template = FLOW_TEMPLATES.find((entry) => entry.id === templateId);
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
        const insertion = insertStepRelativeToTarget(
          current,
          selectedId,
          "after",
          draftStep,
        );
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
    placeNodeAtCursor(draftStep.id, options?.cursorPosition ?? null);
    setStatusMessage("Function added to canvas.");
  }

  function addBranchStep(
    conditionStepId: string,
    branch: "then" | "else",
    stepType: DraftWorkflowStep["type"],
  ) {
    pushHistoryCheckpoint();
    const draftStep = createDraftStep(stepType, createId);
    setSteps((current) =>
      appendStepToBranch(current, conditionStepId, branch, draftStep),
    );
    selectStep(draftStep.id);
  }

  function updateSelectedStep(updater: (step: DraftWorkflowStep) => DraftWorkflowStep) {
    if (!selectedStepId) {
      return;
    }

    pushHistoryCheckpoint();
    setSteps((current) => updateStepById(current, selectedStepId, updater));
  }

  function removeSelectedStep() {
    if (!selectedStepId) {
      return;
    }

    pushHistoryCheckpoint();
    setSteps((current) => removeStepById(current, selectedStepId));
    if (connectionDrag?.sourceStepId === selectedStepId) {
      setConnectionDrag(null);
    }
    if (wiringSourceStepId === selectedStepId) {
      setWiringSourceStepId(null);
    }
    setSelectedStepId(null);
    setSelectedCanvasNodeIds([]);
    setInspectorNode("trigger");
    setTimeout(() => {
      ensureAtLeastOneRootStep();
    }, 0);
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

    const draftSteps = transportSteps.map((step) =>
      createDraftFromTransport(step, createId),
    );
    setSteps(draftSteps);
    setConnectionDrag(null);
    setWiringSourceStepId(null);

    const firstStepId = draftSteps.at(0)?.id ?? null;
    setSelectedStepId(firstStepId);
    setSelectedCanvasNodeIds(firstStepId ? [firstStepId] : []);
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
    await saveWorkflow(steps);
  }

  function handleExecutionWorkflowChange(nextWorkflow: string) {
    setSelectedWorkflow(nextWorkflow);
    if (!nextWorkflow) {
      router.push("/maker/automation");
      return;
    }

    router.push(workflowModePath(nextWorkflow, workflowWorkspaceMode));
  }

  return (
    <div className="relative h-[calc(100vh-9rem)] min-h-[760px] overflow-hidden rounded-xl border border-zinc-200 bg-zinc-100">
      <div className="absolute inset-x-0 top-0 z-40 flex flex-wrap items-center gap-2 border-b border-zinc-200 bg-white/95 px-3 py-2 backdrop-blur">
        <StatusBadge tone="neutral">Flows {workflows.length}</StatusBadge>
        <StatusBadge tone="neutral">Runs {runs.length}</StatusBadge>
        <StatusBadge tone="neutral">
          View {workflowWorkspaceMode === "edit" ? "Edit" : "History"}
        </StatusBadge>
        {selectedWorkflow ? (
          <StatusBadge tone="neutral">Active {selectedWorkflow}</StatusBadge>
        ) : null}
        {errorMessage ? (
          <p className="max-w-md truncate rounded-md border border-red-200 bg-red-50 px-2 py-1 text-xs text-red-700">
            {errorMessage}
          </p>
        ) : null}
        {statusMessage ? (
          <p className="max-w-md truncate rounded-md border border-emerald-200 bg-emerald-50 px-2 py-1 text-xs text-emerald-700">
            {statusMessage}
          </p>
        ) : null}
        <div className="ml-auto flex flex-wrap items-center gap-2">
          <Button type="button" size="sm" variant="outline" onClick={undoCanvasEdit} disabled={undoStack.length === 0}>
            Undo
          </Button>
          <Button type="button" size="sm" variant="outline" onClick={redoCanvasEdit} disabled={redoStack.length === 0}>
            Redo
          </Button>
          <Button type="button" size="sm" variant="outline" onClick={() => setSnapToGrid((current) => !current)}>
            Snap {snapToGrid ? "On" : "Off"}
          </Button>
          <Button type="button" size="sm" variant="outline" onClick={resetCanvasLayout}>
            Auto Arrange
          </Button>
          <Button type="button" size="sm" variant="outline" onClick={openNodePicker}>
            Add Node (A)
          </Button>
          <Button type="button" size="sm" variant="outline" onClick={() => setShowBuilderPanel((current) => !current)}>
            {showBuilderPanel ? "Hide Builder" : "Show Builder"}
          </Button>
          <Button type="button" size="sm" variant="outline" onClick={() => setShowInspectorPanel((current) => !current)}>
            {showInspectorPanel ? "Hide Inspector" : "Show Inspector"}
          </Button>
          <Button
            type="button"
            size="sm"
            variant={workflowWorkspaceMode === "edit" ? "default" : "outline"}
            onClick={() => navigateWorkspaceMode("edit")}
          >
            Edit View
          </Button>
          <Button
            type="button"
            size="sm"
            variant={workflowWorkspaceMode === "history" ? "default" : "outline"}
            onClick={() => navigateWorkspaceMode("history")}
          >
            History View
          </Button>
          {wiringSourceStep ? (
            <Button type="button" size="sm" variant="outline" onClick={cancelWireRouting}>
              Cancel Rewire
            </Button>
          ) : null}
        </div>
      </div>

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
        workflowQuery={workflowQuery}
        onWorkflowQueryChange={setWorkflowQuery}
        filteredWorkflows={filteredWorkflows}
        selectedWorkflow={selectedWorkflow}
        onLoadWorkflow={loadWorkflowIntoBuilder}
        onOpenWorkflowHistory={openWorkflowHistory}
        onResetBuilder={resetBuilder}
        workflowWorkspaceMode={workflowWorkspaceMode}
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
        onCatalogInsertModeChange={setCatalogInsertMode}
        canInsertIntoConditionBranch={canInsertIntoConditionBranch}
        filteredTemplates={filteredTemplates}
        onInsertTemplate={insertTemplateFromCatalog}
        onAddRootStep={addRootStep}
        isSaving={isSaving}
        onExecuteWorkflow={handleExecuteWorkflow}
        onExecutionWorkflowChange={handleExecutionWorkflowChange}
        workflows={workflows}
        executePayload={executePayload}
        onExecutePayloadChange={setExecutePayload}
        isExecuting={isExecuting}
        steps={steps}
        selectedStepId={selectedStepId}
        onSelectStep={selectStep}
        onAddBranchStep={addBranchStep}
        selectedWorkflowRuns={selectedWorkflowRuns}
        expandedRunId={expandedRunId}
        attemptsByRun={attemptsByRun}
        onToggleAttempts={(runId) => {
          void toggleAttempts(runId);
        }}
      />

      <WorkflowInspectorPanel
        open={showInspectorPanel}
        inspectorNode={inspectorNode}
        selectedStep={selectedStep}
        triggerType={triggerType}
        triggerEntityLogicalName={triggerEntityLogicalName}
        onTriggerTypeChange={updateTriggerType}
        onTriggerEntityChange={updateTriggerEntity}
        onUpdateSelectedStep={updateSelectedStep}
        onRemoveSelectedStep={removeSelectedStep}
      />

      <WorkflowCanvasScene
        canvasRef={canvasRef}
        canvasSurfaceWidth={canvasSurfaceWidth}
        canvasSurfaceHeight={canvasSurfaceHeight}
        leftPanelOffset={leftPanelOffset}
        rightPanelOffset={rightPanelOffset}
        snapToGrid={snapToGrid}
        selectionBox={selectionBox}
        laneCount={laneCount}
        canvasGraph={canvasGraph}
        nodePositions={nodePositions}
        connectionDrag={connectionDrag}
        steps={steps}
        inspectorNode={inspectorNode}
        selectedStepId={selectedStepId}
        selectedCanvasNodeIds={selectedCanvasNodeIds}
        wiringSourceStepId={wiringSourceStepId}
        onCanvasPointerMove={(event) => {
          const position = pointerToCanvasPosition(event.clientX, event.clientY);
          if (position) {
            lastCanvasPointerRef.current = position;
          }
        }}
        onBeginSelectionBox={beginSelectionBox}
        onSetInspectorTrigger={() => setInspectorNode("trigger")}
        onSetSelectedStepId={setSelectedStepId}
        onSetSelectedCanvasNodeIds={setSelectedCanvasNodeIds}
        onBeginNodeDrag={beginNodeDrag}
        onSelectStep={selectStep}
        onBeginWireDrag={beginWireDrag}
        onStartWireRouting={startWireRouting}
        isHoveredRerouteTarget={isHoveredRerouteTarget}
        onRerouteStep={rerouteStep}
        findStepById={findStepById}
      />
    </div>
  );
}

"use client";

import {
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
  type ExecuteWorkflowRequest,
  type SaveWorkflowRequest,
  type WorkflowConditionOperatorDto,
  type WorkflowResponse,
  type WorkflowRunAttemptResponse,
  type WorkflowRunResponse,
  type WorkflowStepDto,
} from "@/lib/api";
import { formatUtcDateTime } from "@/lib/date-format";
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
  TRIGGER_OPTIONS,
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
  firstActionFromSteps,
  insertStepRelativeToTarget,
  isTypingElement,
  maxCanvasDepth,
  parseJsonObject,
  parseJsonValue,
  removeStepById,
  rerouteTargetFromDataset,
  rerouteTargetsEqual,
  resolveTemplateList,
  stepContainsId,
  summarizeStep,
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

export function WorkflowStudioPanel({
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


  const [logicalName, setLogicalName] = useState("");
  const [displayName, setDisplayName] = useState("");
  const [description, setDescription] = useState("");
  const [triggerType, setTriggerType] = useState<TriggerType>("manual");
  const [triggerEntityLogicalName, setTriggerEntityLogicalName] = useState("");
  const [maxAttempts, setMaxAttempts] = useState("3");
  const [isEnabled, setIsEnabled] = useState(true);
  const [steps, setSteps] = useState<DraftWorkflowStep[]>([
    createDraftStep("log_message", createId),
  ]);

  const [inspectorNode, setInspectorNode] = useState<InspectorNode>("trigger");
  const [selectedStepId, setSelectedStepId] = useState<string | null>(null);

  const initialWorkflowSelection =
    initialSelectedWorkflow === undefined
      ? workflows.at(0)?.logical_name ?? ""
      : workflows.some((workflow) => workflow.logical_name === initialSelectedWorkflow)
        ? initialSelectedWorkflow
        : "";

  const [selectedWorkflow, setSelectedWorkflow] =
    useState(initialWorkflowSelection);
  const [workflowWorkspaceMode, setWorkflowWorkspaceMode] = useState<
    "edit" | "history"
  >(initialWorkspaceMode ?? "edit");
  const [workflowQuery, setWorkflowQuery] = useState("");
  const [executePayload, setExecutePayload] = useState(
    JSON.stringify({ manual: true }, null, 2),
  );

  const [attemptsByRun, setAttemptsByRun] = useState<
    Record<string, WorkflowRunAttemptResponse[]>
  >({});
  const [expandedRunId, setExpandedRunId] = useState<string | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);
  const [isSaving, setIsSaving] = useState(false);
  const [isExecuting, setIsExecuting] = useState(false);
  const [showBuilderPanel, setShowBuilderPanel] = useState(true);
  const [showInspectorPanel, setShowInspectorPanel] = useState(true);
  const [catalogQuery, setCatalogQuery] = useState("");
  const [catalogCategory, setCatalogCategory] =
    useState<"all" | FlowTemplateCategory>("all");
  const [catalogInsertMode, setCatalogInsertMode] =
    useState<CatalogInsertMode>("after_selected");
  const [showNodePicker, setShowNodePicker] = useState(false);
  const [nodePickerQuery, setNodePickerQuery] = useState("");
  const [nodePickerCategory, setNodePickerCategory] =
    useState<"all" | FlowTemplateCategory>("all");
  const [nodePickerInsertMode, setNodePickerInsertMode] =
    useState<CatalogInsertMode>("after_selected");
  const [snapToGrid, setSnapToGrid] = useState(true);
  const [wiringSourceStepId, setWiringSourceStepId] = useState<string | null>(null);
  const [selectedCanvasNodeIds, setSelectedCanvasNodeIds] = useState<string[]>([]);
  const [selectionBox, setSelectionBox] = useState<SelectionBoxState | null>(null);
  const [connectionDrag, setConnectionDrag] = useState<{
    sourceStepId: string;
    pointerX: number;
    pointerY: number;
    hoveredTarget: RerouteTarget | null;
  } | null>(null);
  const [nodePositions, setNodePositions] = useState<Record<string, CanvasPosition>>({});
  const [undoStack, setUndoStack] = useState<CanvasHistorySnapshot[]>([]);
  const [redoStack, setRedoStack] = useState<CanvasHistorySnapshot[]>([]);
  const [dragState, setDragState] = useState<{
    nodeIds: string[];
    pointerStartX: number;
    pointerStartY: number;
    initialPositions: Record<string, CanvasPosition>;
    hasMoved: boolean;
  } | null>(null);

  const canvasRef = useRef<HTMLDivElement | null>(null);
  const nodePickerInputRef = useRef<HTMLInputElement | null>(null);
  const lastCanvasPointerRef = useRef<CanvasPosition | null>(null);
  const suppressHistoryRef = useRef(false);
  const initializedFromRouteRef = useRef(false);

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

  function pointerToCanvasPosition(clientX: number, clientY: number): CanvasPosition | null {
    const canvasElement = canvasRef.current;
    if (!canvasElement) {
      return null;
    }

    const rect = canvasElement.getBoundingClientRect();
    return {
      x: clientX - rect.left + canvasElement.scrollLeft,
      y: clientY - rect.top + canvasElement.scrollTop,
    };
  }

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
  }, [wiringSourceStepId, wiringSourceStep]);

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
  }, [canvasGraph.nodes, steps]);

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
        pushHistoryCheckpoint();
        setDragState((current) =>
          current ? { ...current, hasMoved: true } : current,
        );
      }

      const maxX = Math.max(
        CANVAS_PADDING,
        canvasElement.scrollWidth - CANVAS_NODE_WIDTH - CANVAS_PADDING,
      );
      const maxY = Math.max(
        CANVAS_PADDING,
        canvasElement.scrollHeight - CANVAS_NODE_HEIGHT - CANVAS_PADDING,
      );

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

    function handlePointerUp() {
      setDragState(null);
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

    function handlePointerUp(event: PointerEvent) {
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

      setSelectionBox((current) =>
        current
          ? {
              ...current,
              currentX: pointerPosition.x,
              currentY: pointerPosition.y,
            }
          : current,
      );
    }

    function handlePointerUp() {
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

            return !(nodeRight < left || nodeLeft > right || nodeBottom < top || nodeTop > bottom);
          })
          .map((node) => node.id)
          .filter((nodeId) => nodeId !== TRIGGER_NODE_ID);

        setSelectedCanvasNodeIds(selectedIds);
        return null;
      });
    }

    window.addEventListener("pointermove", handlePointerMove);
    window.addEventListener("pointerup", handlePointerUp);

    return () => {
      window.removeEventListener("pointermove", handlePointerMove);
      window.removeEventListener("pointerup", handlePointerUp);
    };
  }, [selectionBox, canvasGraph.nodes, nodePositions]);

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
  }, [showNodePicker]);

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

  function compileStep(step: DraftWorkflowStep): WorkflowStepDto {
    if (step.type === "log_message") {
      if (step.message.trim().length === 0) {
        throw new Error("Log message step requires a non-empty message.");
      }

      return {
        type: "log_message",
        message: step.message,
      };
    }

    if (step.type === "create_runtime_record") {
      if (step.entityLogicalName.trim().length === 0) {
        throw new Error("Create record step requires an entity logical name.");
      }

      return {
        type: "create_runtime_record",
        entity_logical_name: step.entityLogicalName,
        data: parseJsonObject(step.dataJson, "Create record step data"),
      };
    }

    if (step.fieldPath.trim().length === 0) {
      throw new Error("Condition step requires a payload field path.");
    }

    const thenSteps = step.thenSteps.map(compileStep);
    const elseSteps = step.elseSteps.map(compileStep);
    if (thenSteps.length === 0 && elseSteps.length === 0) {
      throw new Error("Condition step requires at least one branch step.");
    }

    const value =
      step.operator === "exists"
        ? null
        : (parseJsonValue(step.valueJson, "Condition value") as unknown);

    return {
      type: "condition",
      field_path: step.fieldPath,
      operator: step.operator,
      value,
      then_label: step.thenLabel.trim().length > 0 ? step.thenLabel.trim() : null,
      else_label: step.elseLabel.trim().length > 0 ? step.elseLabel.trim() : null,
      then_steps: thenSteps,
      else_steps: elseSteps,
    };
  }

  async function handleSaveWorkflow(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    resetMessages();
    setIsSaving(true);

    try {
      const parsedMaxAttempts = Number.parseInt(maxAttempts, 10);
      if (!Number.isFinite(parsedMaxAttempts)) {
        throw new Error("Max attempts must be a number.");
      }

      const compiledSteps = steps.map(compileStep);
      if (compiledSteps.length === 0) {
        throw new Error("Flow canvas requires at least one step.");
      }

      const firstAction = firstActionFromSteps(compiledSteps);
      if (!firstAction) {
        throw new Error("Flow canvas must contain at least one executable action step.");
      }

      const payload: SaveWorkflowRequest = {
        logical_name: logicalName,
        display_name: displayName,
        description: description.trim().length > 0 ? description : null,
        trigger_type: triggerType,
        trigger_entity_logical_name:
          triggerType === "runtime_record_created" &&
          triggerEntityLogicalName.trim().length > 0
            ? triggerEntityLogicalName
            : null,
        action_type: firstAction.actionType,
        action_entity_logical_name: firstAction.actionEntityLogicalName,
        action_payload: firstAction.actionPayload,
        steps: compiledSteps,
        max_attempts: parsedMaxAttempts,
        is_enabled: isEnabled,
      };

      const response = await apiFetch("/api/workflows", {
        method: "POST",
        body: JSON.stringify(payload),
      });

      if (!response.ok) {
        const payload = (await response.json()) as { message?: string };
        setErrorMessage(payload.message ?? "Unable to save workflow.");
        return;
      }

      setStatusMessage("Workflow saved.");
      router.refresh();
    } catch (error) {
      setErrorMessage(
        error instanceof Error ? error.message : "Unable to save workflow.",
      );
    } finally {
      setIsSaving(false);
    }
  }

  async function handleExecuteWorkflow(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!selectedWorkflow) {
      setErrorMessage("Select a workflow first.");
      return;
    }

    resetMessages();
    setIsExecuting(true);
    try {
      const triggerPayload = parseJsonObject(executePayload, "Trigger payload");
      const payload: ExecuteWorkflowRequest = {
        trigger_payload: triggerPayload,
      };

      const response = await apiFetch(`/api/workflows/${selectedWorkflow}/execute`, {
        method: "POST",
        body: JSON.stringify(payload),
      });

      if (!response.ok) {
        const payload = (await response.json()) as { message?: string };
        setErrorMessage(payload.message ?? "Unable to execute workflow.");
        return;
      }

      setStatusMessage("Workflow executed.");
      router.refresh();
    } catch (error) {
      setErrorMessage(
        error instanceof Error ? error.message : "Unable to execute workflow.",
      );
    } finally {
      setIsExecuting(false);
    }
  }

  async function toggleAttempts(runId: string) {
    if (expandedRunId === runId) {
      setExpandedRunId(null);
      return;
    }

    if (!attemptsByRun[runId]) {
      const response = await apiFetch(`/api/workflows/runs/${runId}/attempts`);
      if (!response.ok) {
        setErrorMessage("Unable to load workflow run attempts.");
        return;
      }

      const attempts = (await response.json()) as WorkflowRunAttemptResponse[];
      setAttemptsByRun((current) => ({ ...current, [runId]: attempts }));
    }

    setExpandedRunId(runId);
  }

  function renderCanvasStep(
    step: DraftWorkflowStep,
    depth: number,
    branchLabel?: string,
  ) {
    const isSelected = selectedStepId === step.id;

    if (step.type !== "condition") {
      return (
        <div key={step.id} className="space-y-2">
          {branchLabel ? (
            <p className="text-[11px] font-semibold uppercase tracking-wide text-zinc-500">
              {branchLabel}
            </p>
          ) : null}
          <button
            type="button"
            className={`w-full rounded-lg border p-3 text-left transition ${
              isSelected
                ? "border-emerald-500 bg-emerald-50"
                : "border-zinc-200 bg-white hover:border-emerald-300"
            }`}
            style={{ marginLeft: `${depth * 14}px` }}
            onClick={() => selectStep(step.id)}
          >
            <p className="text-xs font-semibold uppercase tracking-wide text-emerald-700">
              {step.type.replaceAll("_", " ")}
            </p>
            <p className="mt-1 text-sm text-zinc-900">{summarizeStep(step)}</p>
          </button>
        </div>
      );
    }

    return (
      <div key={step.id} className="space-y-2" style={{ marginLeft: `${depth * 14}px` }}>
        <button
          type="button"
          className={`w-full rounded-lg border p-3 text-left transition ${
            isSelected
              ? "border-emerald-500 bg-emerald-50"
              : "border-zinc-200 bg-white hover:border-emerald-300"
          }`}
          onClick={() => selectStep(step.id)}
        >
          <p className="text-xs font-semibold uppercase tracking-wide text-emerald-700">
            Condition
          </p>
          <p className="mt-1 text-sm text-zinc-900">{summarizeStep(step)}</p>
        </button>

        <div className="grid gap-2 md:grid-cols-2">
          <div className="space-y-2 rounded-md border border-emerald-100 bg-white p-2">
            <p className="text-[11px] font-semibold uppercase tracking-wide text-emerald-700">
              If {step.thenLabel || "Yes"}
            </p>
            {step.thenSteps.map((childStep) => renderCanvasStep(childStep, depth + 1))}
            <div className="flex flex-wrap gap-1">
              <Button
                type="button"
                size="sm"
                variant="outline"
                onClick={() => addBranchStep(step.id, "then", "log_message")}
              >
                + Log
              </Button>
              <Button
                type="button"
                size="sm"
                variant="outline"
                onClick={() => addBranchStep(step.id, "then", "create_runtime_record")}
              >
                + Create
              </Button>
              <Button
                type="button"
                size="sm"
                variant="outline"
                onClick={() => addBranchStep(step.id, "then", "condition")}
              >
                + Condition
              </Button>
            </div>
          </div>

          <div className="space-y-2 rounded-md border border-zinc-200 bg-white p-2">
            <p className="text-[11px] font-semibold uppercase tracking-wide text-zinc-600">
              If {step.elseLabel || "No"}
            </p>
            {step.elseSteps.map((childStep) => renderCanvasStep(childStep, depth + 1))}
            <div className="flex flex-wrap gap-1">
              <Button
                type="button"
                size="sm"
                variant="outline"
                onClick={() => addBranchStep(step.id, "else", "log_message")}
              >
                + Log
              </Button>
              <Button
                type="button"
                size="sm"
                variant="outline"
                onClick={() => addBranchStep(step.id, "else", "create_runtime_record")}
              >
                + Create
              </Button>
              <Button
                type="button"
                size="sm"
                variant="outline"
                onClick={() => addBranchStep(step.id, "else", "condition")}
              >
                + Condition
              </Button>
            </div>
          </div>
        </div>
      </div>
    );
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

      {showNodePicker ? (
        <div className="absolute inset-0 z-50 flex items-start justify-center bg-zinc-900/35 pt-24" onClick={closeNodePicker}>
          <div
            className="w-[min(760px,calc(100%-2rem))] rounded-xl border border-zinc-200 bg-white shadow-2xl"
            onClick={(event) => event.stopPropagation()}
          >
            <div className="border-b border-zinc-200 p-3">
              <p className="text-xs font-semibold uppercase tracking-wide text-zinc-600">
                Node Picker
              </p>
              <Input
                ref={nodePickerInputRef}
                value={nodePickerQuery}
                onChange={(event) => setNodePickerQuery(event.target.value)}
                placeholder="Search functions..."
              />
              <div className="mt-2 grid grid-cols-2 gap-2">
                <Select
                  value={nodePickerCategory}
                  onChange={(event) =>
                    setNodePickerCategory(
                      event.target.value as "all" | FlowTemplateCategory,
                    )
                  }
                >
                  <option value="all">All</option>
                  <option value="trigger">Trigger</option>
                  <option value="logic">Logic</option>
                  <option value="integration">Integration</option>
                  <option value="data">Data</option>
                  <option value="operations">Operations</option>
                </Select>

                <Select
                  value={nodePickerInsertMode}
                  onChange={(event) =>
                    setNodePickerInsertMode(event.target.value as CatalogInsertMode)
                  }
                >
                  <option value="after_selected">After selected</option>
                  <option value="root">Append root</option>
                  <option value="then_selected" disabled={!canInsertIntoConditionBranch}>
                    Condition: yes
                  </option>
                  <option value="else_selected" disabled={!canInsertIntoConditionBranch}>
                    Condition: no
                  </option>
                </Select>
              </div>
            </div>

            <div className="max-h-[420px] space-y-2 overflow-y-auto p-3">
              {nodePickerTemplates.length > 0 ? (
                nodePickerTemplates.map((template, index) => (
                  <button
                    key={template.id}
                    type="button"
                    className={`w-full rounded-md border px-3 py-2 text-left transition ${
                      index === 0
                        ? "border-emerald-400 bg-emerald-50"
                        : "border-zinc-200 bg-white hover:border-emerald-300"
                    }`}
                    onClick={() => insertFromNodePicker(template.id)}
                  >
                    <p className="text-sm font-semibold text-zinc-900">
                      {template.label}
                      <span className="ml-2 text-[10px] uppercase tracking-wide text-zinc-500">
                        {template.category}
                      </span>
                    </p>
                    <p className="text-xs text-zinc-600">{template.description}</p>
                  </button>
                ))
              ) : (
                <p className="text-sm text-zinc-500">No functions match your search.</p>
              )}
            </div>

            <div className="flex items-center justify-between border-t border-zinc-200 px-3 py-2 text-xs text-zinc-600">
              <span>`A` open picker • `Enter` insert top match • `Esc` close</span>
              <Button type="button" size="sm" variant="outline" onClick={closeNodePicker}>
                Close
              </Button>
            </div>
          </div>
        </div>
      ) : null}

      {showBuilderPanel ? (
        <div className="absolute bottom-3 left-3 top-16 z-30 w-[340px] overflow-y-auto rounded-lg border border-zinc-200 bg-white/95 p-3 shadow-lg backdrop-blur">
          <div className="space-y-3">
            <div className="space-y-2 rounded-md border border-zinc-200 bg-zinc-50 p-2">
              <p className="text-[11px] font-semibold uppercase tracking-wide text-zinc-600">
                Workflow Library
              </p>
              <Input
                value={workflowQuery}
                onChange={(event) => setWorkflowQuery(event.target.value)}
                placeholder="Search workflows"
              />
              <div className="max-h-40 space-y-1 overflow-y-auto pr-1">
                {filteredWorkflows.length > 0 ? (
                  filteredWorkflows.map((workflow) => {
                    const isSelected = selectedWorkflow === workflow.logical_name;
                    return (
                      <div
                        key={workflow.logical_name}
                        className={`rounded-md border px-2 py-2 ${
                          isSelected
                            ? "border-emerald-300 bg-emerald-50"
                            : "border-zinc-200 bg-white"
                        }`}
                      >
                        <p className="text-xs font-semibold text-zinc-900">
                          {workflow.display_name}
                        </p>
                        <p className="font-mono text-[10px] text-zinc-500">
                          {workflow.logical_name}
                        </p>
                        <div className="mt-2 flex gap-1">
                          <Button
                            type="button"
                            size="sm"
                            variant="outline"
                            onClick={() => loadWorkflowIntoBuilder(workflow)}
                          >
                            Edit
                          </Button>
                          <Button
                            type="button"
                            size="sm"
                            variant="outline"
                            onClick={() => openWorkflowHistory(workflow)}
                          >
                            History
                          </Button>
                        </div>
                      </div>
                    );
                  })
                ) : (
                  <p className="text-[11px] text-zinc-500">No matching workflows.</p>
                )}
              </div>
              <Button type="button" size="sm" variant="outline" onClick={resetBuilder}>
                New Workflow
              </Button>
            </div>

            {workflowWorkspaceMode === "edit" ? (
              <>
                <form className="space-y-3" onSubmit={handleSaveWorkflow}>
                  <p className="text-xs font-semibold uppercase tracking-wide text-zinc-600">
                    Flow Builder
                  </p>
                  <Input
                    value={logicalName}
                    onChange={(event) => setLogicalName(event.target.value)}
                    placeholder="logical_name"
                    required
                  />
                  <Input
                    value={displayName}
                    onChange={(event) => setDisplayName(event.target.value)}
                    placeholder="Display name"
                    required
                  />
                  <Input
                    value={description}
                    onChange={(event) => setDescription(event.target.value)}
                    placeholder="Description"
                  />
                  <Input
                    type="number"
                    min={1}
                    max={10}
                    value={maxAttempts}
                    onChange={(event) => setMaxAttempts(event.target.value)}
                    required
                  />
                  <label className="inline-flex items-center gap-2 text-xs text-zinc-700">
                    <input
                      type="checkbox"
                      checked={isEnabled}
                      onChange={(event) => setIsEnabled(event.target.checked)}
                    />
                    enabled
                  </label>

                  <div className="space-y-2 rounded-md border border-zinc-200 bg-zinc-50 p-2">
                    <p className="text-[11px] font-semibold uppercase tracking-wide text-zinc-600">
                      Function Catalog
                    </p>
                    <Input
                      value={catalogQuery}
                      onChange={(event) => setCatalogQuery(event.target.value)}
                      placeholder="Search functions"
                    />
                    <div className="grid grid-cols-2 gap-2">
                      <Select
                        value={catalogCategory}
                        onChange={(event) =>
                          setCatalogCategory(
                            event.target.value as "all" | FlowTemplateCategory,
                          )
                        }
                      >
                        <option value="all">All</option>
                        <option value="trigger">Trigger</option>
                        <option value="logic">Logic</option>
                        <option value="integration">Integration</option>
                        <option value="data">Data</option>
                        <option value="operations">Operations</option>
                      </Select>

                      <Select
                        value={catalogInsertMode}
                        onChange={(event) =>
                          setCatalogInsertMode(event.target.value as CatalogInsertMode)
                        }
                      >
                        <option value="after_selected">After selected</option>
                        <option value="root">Append root</option>
                        <option
                          value="then_selected"
                          disabled={!canInsertIntoConditionBranch}
                        >
                          Condition: yes
                        </option>
                        <option
                          value="else_selected"
                          disabled={!canInsertIntoConditionBranch}
                        >
                          Condition: no
                        </option>
                      </Select>
                    </div>

                    <div className="max-h-52 space-y-1 overflow-y-auto pr-1">
                      {filteredTemplates.length > 0 ? (
                        filteredTemplates.map((template) => (
                          <button
                            key={template.id}
                            type="button"
                            className="w-full rounded-md border border-zinc-200 bg-white px-2 py-2 text-left transition hover:border-emerald-300"
                            onClick={() => insertTemplateFromCatalog(template.id)}
                          >
                            <p className="text-xs font-semibold text-zinc-900">
                              {template.label}
                            </p>
                            <p className="text-[11px] text-zinc-600">
                              {template.description}
                            </p>
                          </button>
                        ))
                      ) : (
                        <p className="text-[11px] text-zinc-500">No matching functions.</p>
                      )}
                    </div>

                    <div className="flex flex-wrap gap-1">
                      {STEP_LIBRARY.map((entry) => (
                        <Button
                          key={entry.type}
                          type="button"
                          size="sm"
                          variant="outline"
                          onClick={() => addRootStep(entry.type)}
                        >
                          + {entry.label}
                        </Button>
                      ))}
                    </div>
                  </div>
                  <Button type="submit" disabled={isSaving}>
                    {isSaving ? "Saving..." : "Save Flow"}
                  </Button>
                </form>

                <Separator className="my-3" />

                <form className="space-y-2" onSubmit={handleExecuteWorkflow}>
                  <p className="text-xs font-semibold uppercase tracking-wide text-zinc-600">
                    Test Run
                  </p>
                  <Select
                    value={selectedWorkflow}
                    onChange={(event) => {
                      const nextWorkflow = event.target.value;
                      setSelectedWorkflow(nextWorkflow);
                      if (!nextWorkflow) {
                        router.push("/maker/automation");
                        return;
                      }

                      router.push(
                        workflowModePath(nextWorkflow, workflowWorkspaceMode),
                      );
                    }}
                  >
                    <option value="">Select workflow</option>
                    {workflows.map((workflow) => (
                      <option key={workflow.logical_name} value={workflow.logical_name}>
                        {workflow.display_name}
                      </option>
                    ))}
                  </Select>
                  <Textarea
                    className="font-mono text-xs"
                    value={executePayload}
                    onChange={(event) => setExecutePayload(event.target.value)}
                    rows={4}
                  />
                  <Button
                    type="submit"
                    size="sm"
                    variant="outline"
                    disabled={isExecuting}
                  >
                    {isExecuting ? "Executing..." : "Execute"}
                  </Button>
                </form>

                <Separator className="my-3" />
                <div className="space-y-2">
                  <p className="text-xs font-semibold uppercase tracking-wide text-zinc-600">
                    Flow Outline
                  </p>
                  <div className="space-y-2">{steps.map((step) => renderCanvasStep(step, 0))}</div>
                </div>
              </>
            ) : (
              <div className="space-y-2">
                <p className="text-xs font-semibold uppercase tracking-wide text-zinc-600">
                  Workflow History
                </p>
                {selectedWorkflowRuns.length > 0 ? (
                  <div className="max-h-[520px] space-y-2 overflow-y-auto pr-1">
                    {selectedWorkflowRuns.map((run) => (
                      <div key={run.run_id} className="rounded-md border border-zinc-200 p-2 text-xs">
                        <div className="flex flex-wrap items-center gap-2">
                          <span className="font-mono text-[11px]">{run.run_id}</span>
                          <StatusBadge tone="neutral">{run.status}</StatusBadge>
                          <span>{formatUtcDateTime(run.started_at)}</span>
                          <Button
                            type="button"
                            size="sm"
                            variant="outline"
                            onClick={() => {
                              void toggleAttempts(run.run_id);
                            }}
                          >
                            {expandedRunId === run.run_id ? "Hide" : "Attempts"}
                          </Button>
                        </div>
                        {expandedRunId === run.run_id ? (
                          <div className="mt-1 space-y-1 text-[11px] text-zinc-600">
                            {(attemptsByRun[run.run_id] ?? []).map((attempt) => (
                              <p key={`${attempt.run_id}-${attempt.attempt_number}`}>
                                #{attempt.attempt_number} {attempt.status}
                                {attempt.error_message ? ` - ${attempt.error_message}` : ""}
                              </p>
                            ))}
                          </div>
                        ) : null}
                      </div>
                    ))}
                  </div>
                ) : (
                  <p className="text-xs text-zinc-500">
                    {selectedWorkflow
                      ? "No runs for this workflow yet."
                      : "Select a workflow to view run history."}
                  </p>
                )}
              </div>
            )}
          </div>
        </div>
      ) : null}

      {showInspectorPanel ? (
        <div className="absolute bottom-3 right-3 top-16 z-30 w-[360px] overflow-y-auto rounded-lg border border-zinc-200 bg-white/95 p-3 shadow-lg backdrop-blur">
          <p className="text-xs font-semibold uppercase tracking-wide text-zinc-600">
            Inspector {inspectorNode === "trigger" ? "Trigger" : "Step"}
          </p>
          <div className="mt-3 space-y-3">
            {inspectorNode === "trigger" ? (
              <>
                <div className="space-y-2">
                  <Label htmlFor="workflow_trigger_type">Trigger Type</Label>
                  <Select id="workflow_trigger_type" value={triggerType} onChange={(event) => updateTriggerType(event.target.value as TriggerType)}>
                    {TRIGGER_OPTIONS.map((option) => (
                      <option key={option.value} value={option.value}>
                        {option.value}
                      </option>
                    ))}
                  </Select>
                </div>
                <div className="space-y-2">
                  <Label htmlFor="workflow_trigger_entity">Trigger Entity</Label>
                  <Input id="workflow_trigger_entity" value={triggerEntityLogicalName} onChange={(event) => updateTriggerEntity(event.target.value)} placeholder="contact" disabled={triggerType !== "runtime_record_created"} />
                </div>
              </>
            ) : selectedStep ? (
              <>
                {selectedStep.type === "log_message" ? (
                  <div className="space-y-2">
                    <Label htmlFor="workflow_step_message">Message</Label>
                    <Input id="workflow_step_message" value={selectedStep.message} onChange={(event) => updateSelectedStep((step) => (step.type === "log_message" ? { ...step, message: event.target.value } : step))} />
                  </div>
                ) : null}

                {selectedStep.type === "create_runtime_record" ? (
                  <>
                    <div className="space-y-2">
                      <Label htmlFor="workflow_step_entity">Entity Logical Name</Label>
                      <Input id="workflow_step_entity" value={selectedStep.entityLogicalName} onChange={(event) => updateSelectedStep((step) => (step.type === "create_runtime_record" ? { ...step, entityLogicalName: event.target.value } : step))} />
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="workflow_step_data">Data JSON</Label>
                      <Textarea id="workflow_step_data" className="font-mono text-xs" rows={8} value={selectedStep.dataJson} onChange={(event) => updateSelectedStep((step) => (step.type === "create_runtime_record" ? { ...step, dataJson: event.target.value } : step))} />
                    </div>
                  </>
                ) : null}

                {selectedStep.type === "condition" ? (
                  <>
                    <Input value={selectedStep.fieldPath} onChange={(event) => updateSelectedStep((step) => (step.type === "condition" ? { ...step, fieldPath: event.target.value } : step))} placeholder="field path" />
                    <Select value={selectedStep.operator} onChange={(event) => updateSelectedStep((step) => (step.type === "condition" ? { ...step, operator: event.target.value as WorkflowConditionOperatorDto } : step))}>
                      {CONDITION_OPERATORS.map((operator) => (
                        <option key={operator} value={operator}>{operator}</option>
                      ))}
                    </Select>
                    <Input value={selectedStep.valueJson} disabled={selectedStep.operator === "exists"} onChange={(event) => updateSelectedStep((step) => (step.type === "condition" ? { ...step, valueJson: event.target.value } : step))} placeholder='"open"' />
                    <div className="grid grid-cols-2 gap-2">
                      <Input value={selectedStep.thenLabel} onChange={(event) => updateSelectedStep((step) => (step.type === "condition" ? { ...step, thenLabel: event.target.value } : step))} placeholder="Yes label" />
                      <Input value={selectedStep.elseLabel} onChange={(event) => updateSelectedStep((step) => (step.type === "condition" ? { ...step, elseLabel: event.target.value } : step))} placeholder="No label" />
                    </div>
                  </>
                ) : null}

                <Button type="button" size="sm" variant="outline" onClick={removeSelectedStep}>
                  Remove Selected Step
                </Button>
              </>
            ) : (
              <p className="text-sm text-zinc-500">Select any node on canvas to edit.</p>
            )}
          </div>
        </div>
      ) : null}

      <div
        ref={canvasRef}
        className="absolute inset-0 overflow-auto pt-16"
        onPointerMove={(event) => {
          const position = pointerToCanvasPosition(event.clientX, event.clientY);
          if (position) {
            lastCanvasPointerRef.current = position;
          }
        }}
      >
        <div
          className="relative"
          style={{
            width: `${canvasSurfaceWidth}px`,
            height: `${canvasSurfaceHeight}px`,
            marginLeft: `${leftPanelOffset}px`,
            marginRight: `${rightPanelOffset}px`,
            backgroundImage: snapToGrid
              ? "linear-gradient(to right, rgba(148,163,184,0.18) 1px, transparent 1px), linear-gradient(to bottom, rgba(148,163,184,0.18) 1px, transparent 1px)"
              : undefined,
            backgroundSize: snapToGrid ? `${GRID_SIZE}px ${GRID_SIZE}px` : undefined,
          }}
          onPointerDown={beginSelectionBox}
        >
          <div className="pointer-events-none absolute inset-0">
            {Array.from({ length: laneCount }, (_, laneIndex) => {
              const left = 220 + laneIndex * LANE_WIDTH;
              const isEven = laneIndex % 2 === 0;
              return (
                <div key={`lane_${laneIndex}`} className={`absolute top-0 h-full border-l border-zinc-200 ${isEven ? "bg-emerald-50/35" : "bg-white/55"}`} style={{ width: `${LANE_WIDTH}px`, left: `${left}px` }}>
                  <p className="px-3 pt-2 text-[10px] font-semibold uppercase tracking-wide text-zinc-500">Stage {laneIndex + 1}</p>
                </div>
              );
            })}
          </div>

          {selectionBox ? (
            <div className="pointer-events-none absolute rounded-sm border border-sky-400 bg-sky-200/20" style={{ left: `${Math.min(selectionBox.startX, selectionBox.currentX)}px`, top: `${Math.min(selectionBox.startY, selectionBox.currentY)}px`, width: `${Math.abs(selectionBox.currentX - selectionBox.startX)}px`, height: `${Math.abs(selectionBox.currentY - selectionBox.startY)}px` }} />
          ) : null}

          <svg className="pointer-events-none absolute inset-0" width={canvasSurfaceWidth} height={canvasSurfaceHeight}>
            {canvasGraph.edges.map((edge) => {
              const fromPosition = nodePositions[edge.from];
              const toPosition = nodePositions[edge.to];
              if (!fromPosition || !toPosition) {
                return null;
              }

              const startX = fromPosition.x + CANVAS_NODE_WIDTH;
              const startY = fromPosition.y + CANVAS_NODE_HEIGHT / 2;
              const endX = toPosition.x;
              const endY = toPosition.y + CANVAS_NODE_HEIGHT / 2;
              const travelX = endX - startX;
              const bendX = travelX >= 0 ? startX + Math.max(48, travelX * 0.45) : startX + 72;
              const pathData = `M ${startX} ${startY} L ${bendX} ${startY} L ${bendX} ${endY} L ${endX} ${endY}`;

              return (
                <g key={edge.id}>
                  <path d={pathData} fill="none" stroke="#4ade80" strokeWidth="2" strokeLinejoin="round" />
                  {edge.label ? (
                    <text x={(startX + endX) / 2} y={(startY + endY) / 2 - 8} textAnchor="middle" className="fill-zinc-500 text-[10px] font-semibold uppercase tracking-wide">
                      {edge.label}
                    </text>
                  ) : null}
                </g>
              );
            })}

            {connectionDrag ? (() => {
              const sourcePosition = nodePositions[connectionDrag.sourceStepId];
              if (!sourcePosition) {
                return null;
              }

              const startX = sourcePosition.x + CANVAS_NODE_WIDTH;
              const startY = sourcePosition.y + CANVAS_NODE_HEIGHT / 2;
              const endX = connectionDrag.pointerX;
              const endY = connectionDrag.pointerY;
              const travelX = endX - startX;
              const bendX = travelX >= 0 ? startX + Math.max(48, travelX * 0.45) : startX + 72;
              const pathData = `M ${startX} ${startY} L ${bendX} ${startY} L ${bendX} ${endY} L ${endX} ${endY}`;

              return <path d={pathData} fill="none" stroke="#0284c7" strokeWidth="2" strokeDasharray="6 4" strokeLinejoin="round" />;
            })() : null}
          </svg>

          {canvasGraph.nodes.map((node) => {
            const position = nodePositions[node.id] ?? { x: CANVAS_PADDING, y: CANVAS_PADDING };
            const stepForNode = node.id === TRIGGER_NODE_ID ? null : findStepById(steps, node.id);
            const isSelected = node.id === TRIGGER_NODE_ID ? inspectorNode === "trigger" : selectedStepId === node.id;
            const isMultiSelected = selectedCanvasNodeIds.includes(node.id);
            const isWireSource = wiringSourceStepId === node.id;
            const isWireTarget = Boolean(wiringSourceStepId && !isWireSource);
            const toneClasses = node.tone === "trigger" ? "border-emerald-300 bg-emerald-50" : node.tone === "condition" ? "border-amber-300 bg-amber-50" : "border-sky-300 bg-sky-50";

            return (
              <div key={node.id} className="absolute" data-canvas-node="true" data-canvas-node-id={node.id} style={{ left: `${position.x}px`, top: `${position.y}px`, width: `${CANVAS_NODE_WIDTH}px`, height: `${CANVAS_NODE_HEIGHT}px` }}>
                <button
                  type="button"
                  className={`h-full w-full rounded-lg border p-3 text-left shadow-sm transition ${toneClasses} ${isSelected ? "ring-2 ring-emerald-400" : "hover:ring-2 hover:ring-emerald-200"} ${isWireSource ? "ring-2 ring-sky-500" : ""} ${isMultiSelected && !isSelected ? "ring-2 ring-sky-300" : ""}`}
                  onPointerDown={(event) => beginNodeDrag(node.id, event, event.shiftKey)}
                  onClick={(event) => {
                    if (node.kind === "trigger") {
                      setInspectorNode("trigger");
                      setSelectedStepId(null);
                      setSelectedCanvasNodeIds([TRIGGER_NODE_ID]);
                      return;
                    }

                    if (connectionDrag) {
                      return;
                    }

                    if (event.shiftKey) {
                      setSelectedCanvasNodeIds((current) => current.includes(node.id) ? current.filter((id) => id !== node.id) : [...current, node.id]);
                      return;
                    }

                    selectStep(node.id);
                  }}
                >
                  <p className="text-[11px] font-semibold uppercase tracking-wide text-zinc-700">{node.title}</p>
                  <p className="mt-1 line-clamp-2 text-xs text-zinc-900">{node.subtitle}</p>
                  <p className="mt-2 text-[10px] text-zinc-500">drag to move</p>
                </button>

                {node.kind === "step" ? (
                  <button
                    type="button"
                    className={`absolute -right-3 top-1/2 h-6 w-6 -translate-y-1/2 rounded-full border bg-white text-xs font-semibold shadow ${isWireSource ? "border-sky-500 text-sky-800" : "border-sky-300 text-sky-700"}`}
                    onPointerDown={(event) => beginWireDrag(node.id, event)}
                    onClick={(event) => {
                      event.preventDefault();
                      event.stopPropagation();
                      startWireRouting(node.id);
                    }}
                    title="Start wire reroute from this step"
                  >
                    {"->"}
                  </button>
                ) : null}

                {isWireTarget ? (
                  <>
                    {node.id === TRIGGER_NODE_ID ? (
                      <button
                        type="button"
                        className={`absolute -top-8 left-1/2 -translate-x-1/2 rounded-md border bg-white px-2 py-1 text-[10px] font-semibold uppercase tracking-wide ${isHoveredRerouteTarget({ kind: "trigger_start" }) ? "border-sky-500 text-sky-800" : "border-emerald-300 text-emerald-700"}`}
                        data-wire-target-kind="trigger_start"
                        onPointerDown={(event) => event.stopPropagation()}
                        onClick={(event) => {
                          event.stopPropagation();
                          rerouteStep({ kind: "trigger_start" });
                        }}
                      >
                        insert first
                      </button>
                    ) : null}

                    {node.id !== TRIGGER_NODE_ID ? (
                      <>
                        <button
                          type="button"
                          className={`absolute -left-9 top-2 rounded-md border bg-white px-2 py-1 text-[10px] font-semibold uppercase tracking-wide ${isHoveredRerouteTarget({ kind: "before", targetId: node.id }) ? "border-sky-500 text-sky-800" : "border-zinc-300 text-zinc-700"}`}
                          data-wire-target-kind="before"
                          data-wire-target-id={node.id}
                          onPointerDown={(event) => event.stopPropagation()}
                          onClick={(event) => {
                            event.stopPropagation();
                            rerouteStep({ kind: "before", targetId: node.id });
                          }}
                        >
                          before
                        </button>
                        <button
                          type="button"
                          className={`absolute -left-9 bottom-2 rounded-md border bg-white px-2 py-1 text-[10px] font-semibold uppercase tracking-wide ${isHoveredRerouteTarget({ kind: "after", targetId: node.id }) ? "border-sky-500 text-sky-800" : "border-zinc-300 text-zinc-700"}`}
                          data-wire-target-kind="after"
                          data-wire-target-id={node.id}
                          onPointerDown={(event) => event.stopPropagation()}
                          onClick={(event) => {
                            event.stopPropagation();
                            rerouteStep({ kind: "after", targetId: node.id });
                          }}
                        >
                          after
                        </button>
                      </>
                    ) : null}

                    {stepForNode?.type === "condition" ? (
                      <>
                        <button
                          type="button"
                          className={`absolute -bottom-8 left-2 rounded-md border bg-white px-2 py-1 text-[10px] font-semibold uppercase tracking-wide ${isHoveredRerouteTarget({ kind: "then", targetId: node.id }) ? "border-sky-500 text-sky-800" : "border-emerald-300 text-emerald-700"}`}
                          data-wire-target-kind="then"
                          data-wire-target-id={node.id}
                          onPointerDown={(event) => event.stopPropagation()}
                          onClick={(event) => {
                            event.stopPropagation();
                            rerouteStep({ kind: "then", targetId: node.id });
                          }}
                        >
                          to {stepForNode.thenLabel || "yes"}
                        </button>
                        <button
                          type="button"
                          className={`absolute -bottom-8 right-2 rounded-md border bg-white px-2 py-1 text-[10px] font-semibold uppercase tracking-wide ${isHoveredRerouteTarget({ kind: "else", targetId: node.id }) ? "border-sky-500 text-sky-800" : "border-zinc-300 text-zinc-700"}`}
                          data-wire-target-kind="else"
                          data-wire-target-id={node.id}
                          onPointerDown={(event) => event.stopPropagation()}
                          onClick={(event) => {
                            event.stopPropagation();
                            rerouteStep({ kind: "else", targetId: node.id });
                          }}
                        >
                          to {stepForNode.elseLabel || "no"}
                        </button>
                      </>
                    ) : null}
                  </>
                ) : null}
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}

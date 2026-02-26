import { useRef, useState } from "react";

import type {
  CanvasHistorySnapshot,
  CanvasPosition,
  CatalogInsertMode,
  FlowTemplateCategory,
  RerouteTarget,
  SelectionBoxState,
} from "@/components/automation/workflow-studio/model";

export function useCanvasState() {
  const [showBuilderPanel, setShowBuilderPanel] = useState(true);
  const [showInspectorPanel, setShowInspectorPanel] = useState(true);
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

  return {
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
  };
}

import type { PointerEvent as ReactPointerEvent, RefObject } from "react";

import { CanvasEdge } from "@/components/automation/workflow-studio/canvas/canvas-edge";
import { CanvasNode } from "@/components/automation/workflow-studio/canvas/canvas-node";
import { WorkflowCanvas } from "@/components/automation/workflow-studio/canvas/workflow-canvas";
import {
  CANVAS_NODE_HEIGHT,
  CANVAS_NODE_WIDTH,
  CANVAS_PADDING,
  GRID_SIZE,
  LANE_WIDTH,
  TRIGGER_NODE_ID,
  type DraftWorkflowStep,
  type InspectorNode,
  type RerouteTarget,
  type SelectionBoxState,
} from "@/components/automation/workflow-studio/model";

type CanvasConnectionDrag = {
  sourceStepId: string;
  pointerX: number;
  pointerY: number;
};

type CanvasGraph = {
  nodes: Array<{
    id: string;
    kind: "trigger" | "step";
    title: string;
    subtitle: string;
    tone: "trigger" | "condition" | "action";
  }>;
  edges: Array<{
    id: string;
    from: string;
    to: string;
    label?: string;
  }>;
};

type WorkflowCanvasSceneProps = {
  canvasRef: RefObject<HTMLDivElement | null>;
  canvasSurfaceWidth: number;
  canvasSurfaceHeight: number;
  leftPanelOffset: number;
  rightPanelOffset: number;
  snapToGrid: boolean;
  selectionBox: SelectionBoxState | null;
  laneCount: number;
  canvasGraph: CanvasGraph;
  nodePositions: Record<string, { x: number; y: number }>;
  connectionDrag: CanvasConnectionDrag | null;
  steps: DraftWorkflowStep[];
  inspectorNode: InspectorNode;
  selectedStepId: string | null;
  selectedCanvasNodeIds: string[];
  wiringSourceStepId: string | null;
  onCanvasPointerMove: (event: ReactPointerEvent<HTMLDivElement>) => void;
  onBeginSelectionBox: (event: ReactPointerEvent<HTMLDivElement>) => void;
  onSetInspectorTrigger: () => void;
  onSetSelectedStepId: (value: string | null) => void;
  onSetSelectedCanvasNodeIds: (
    updater: string[] | ((current: string[]) => string[]),
  ) => void;
  onBeginNodeDrag: (
    nodeId: string,
    event: ReactPointerEvent<HTMLButtonElement>,
    appendSelection: boolean,
  ) => void;
  onSelectStep: (stepId: string) => void;
  onBeginWireDrag: (
    stepId: string,
    event: ReactPointerEvent<HTMLButtonElement>,
  ) => void;
  onStartWireRouting: (stepId: string) => void;
  isHoveredRerouteTarget: (target: RerouteTarget) => boolean;
  onRerouteStep: (target: RerouteTarget) => void;
  findStepById: (steps: DraftWorkflowStep[], stepId: string) => DraftWorkflowStep | null;
};

export function WorkflowCanvasScene({
  canvasRef,
  canvasSurfaceWidth,
  canvasSurfaceHeight,
  leftPanelOffset,
  rightPanelOffset,
  snapToGrid,
  selectionBox,
  laneCount,
  canvasGraph,
  nodePositions,
  connectionDrag,
  steps,
  inspectorNode,
  selectedStepId,
  selectedCanvasNodeIds,
  wiringSourceStepId,
  onCanvasPointerMove,
  onBeginSelectionBox,
  onSetInspectorTrigger,
  onSetSelectedStepId,
  onSetSelectedCanvasNodeIds,
  onBeginNodeDrag,
  onSelectStep,
  onBeginWireDrag,
  onStartWireRouting,
  isHoveredRerouteTarget,
  onRerouteStep,
  findStepById,
}: WorkflowCanvasSceneProps) {
  return (
    <WorkflowCanvas
      canvasRef={canvasRef}
      width={canvasSurfaceWidth}
      height={canvasSurfaceHeight}
      leftOffset={leftPanelOffset}
      rightOffset={rightPanelOffset}
      snapToGrid={snapToGrid}
      gridSize={GRID_SIZE}
      onPointerMove={onCanvasPointerMove}
      onPointerDown={onBeginSelectionBox}
    >
      <div className="pointer-events-none absolute inset-0">
        {Array.from({ length: laneCount }, (_, laneIndex) => {
          const left = 220 + laneIndex * LANE_WIDTH;
          const isEven = laneIndex % 2 === 0;
          return (
            <div
              key={`lane_${laneIndex}`}
              className={`absolute top-0 h-full border-l border-zinc-200 ${
                isEven ? "bg-emerald-50/35" : "bg-white/55"
              }`}
              style={{ width: `${LANE_WIDTH}px`, left: `${left}px` }}
            >
              <p className="px-3 pt-2 text-[10px] font-semibold uppercase tracking-wide text-zinc-500">
                Stage {laneIndex + 1}
              </p>
            </div>
          );
        })}
      </div>

      {selectionBox ? (
        <div
          className="pointer-events-none absolute rounded-sm border border-sky-400 bg-sky-200/20"
          style={{
            left: `${Math.min(selectionBox.startX, selectionBox.currentX)}px`,
            top: `${Math.min(selectionBox.startY, selectionBox.currentY)}px`,
            width: `${Math.abs(selectionBox.currentX - selectionBox.startX)}px`,
            height: `${Math.abs(selectionBox.currentY - selectionBox.startY)}px`,
          }}
        />
      ) : null}

      <svg className="pointer-events-none absolute inset-0" width={canvasSurfaceWidth} height={canvasSurfaceHeight}>
        {canvasGraph.edges.map((edge) => {
          const fromPosition = nodePositions[edge.from];
          const toPosition = nodePositions[edge.to];
          if (!fromPosition || !toPosition) {
            return null;
          }

          return (
            <CanvasEdge
              key={edge.id}
              fromX={fromPosition.x + CANVAS_NODE_WIDTH}
              fromY={fromPosition.y + CANVAS_NODE_HEIGHT / 2}
              toX={toPosition.x}
              toY={toPosition.y + CANVAS_NODE_HEIGHT / 2}
              label={edge.label}
            />
          );
        })}

        {connectionDrag
          ? (() => {
              const sourcePosition = nodePositions[connectionDrag.sourceStepId];
              if (!sourcePosition) {
                return null;
              }

              return (
                <CanvasEdge
                  fromX={sourcePosition.x + CANVAS_NODE_WIDTH}
                  fromY={sourcePosition.y + CANVAS_NODE_HEIGHT / 2}
                  toX={connectionDrag.pointerX}
                  toY={connectionDrag.pointerY}
                  stroke="#0284c7"
                  dashed
                />
              );
            })()
          : null}
      </svg>

      {canvasGraph.nodes.map((node) => {
        const position = nodePositions[node.id] ?? { x: CANVAS_PADDING, y: CANVAS_PADDING };
        const stepForNode = node.id === TRIGGER_NODE_ID ? null : findStepById(steps, node.id);
        const isSelected = node.id === TRIGGER_NODE_ID ? inspectorNode === "trigger" : selectedStepId === node.id;
        const isMultiSelected = selectedCanvasNodeIds.includes(node.id);
        const isWireSource = wiringSourceStepId === node.id;
        const isWireTarget = Boolean(wiringSourceStepId && !isWireSource);
        const toneClasses =
          node.tone === "trigger"
            ? "border-emerald-300 bg-emerald-50"
            : node.tone === "condition"
              ? "border-amber-300 bg-amber-50"
              : "border-sky-300 bg-sky-50";

        return (
          <CanvasNode
            key={node.id}
            id={node.id}
            x={position.x}
            y={position.y}
            width={CANVAS_NODE_WIDTH}
            height={CANVAS_NODE_HEIGHT}
          >
            <button
              type="button"
              className={`h-full w-full rounded-lg border p-3 text-left shadow-sm transition ${toneClasses} ${isSelected ? "ring-2 ring-emerald-400" : "hover:ring-2 hover:ring-emerald-200"} ${isWireSource ? "ring-2 ring-sky-500" : ""} ${isMultiSelected && !isSelected ? "ring-2 ring-sky-300" : ""}`}
              onPointerDown={(event) => onBeginNodeDrag(node.id, event, event.shiftKey)}
              onClick={(event) => {
                if (node.kind === "trigger") {
                  onSetInspectorTrigger();
                  onSetSelectedStepId(null);
                  onSetSelectedCanvasNodeIds([TRIGGER_NODE_ID]);
                  return;
                }

                if (connectionDrag) {
                  return;
                }

                if (event.shiftKey) {
                  onSetSelectedCanvasNodeIds((current) =>
                    current.includes(node.id)
                      ? current.filter((id) => id !== node.id)
                      : [...current, node.id],
                  );
                  return;
                }

                onSelectStep(node.id);
              }}
            >
              <p className="text-[11px] font-semibold uppercase tracking-wide text-zinc-700">{node.title}</p>
              <p className="mt-1 line-clamp-2 text-xs text-zinc-900">{node.subtitle}</p>
              <p className="mt-2 text-[10px] text-zinc-500">drag to move</p>
            </button>

            {node.kind === "step" ? (
              <button
                type="button"
                className={`absolute -right-3 top-1/2 h-6 w-6 -translate-y-1/2 rounded-full border bg-white text-xs font-semibold shadow ${
                  isWireSource ? "border-sky-500 text-sky-800" : "border-sky-300 text-sky-700"
                }`}
                onPointerDown={(event) => onBeginWireDrag(node.id, event)}
                onClick={(event) => {
                  event.preventDefault();
                  event.stopPropagation();
                  onStartWireRouting(node.id);
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
                    className={`absolute -top-8 left-1/2 -translate-x-1/2 rounded-md border bg-white px-2 py-1 text-[10px] font-semibold uppercase tracking-wide ${
                      isHoveredRerouteTarget({ kind: "trigger_start" })
                        ? "border-sky-500 text-sky-800"
                        : "border-emerald-300 text-emerald-700"
                    }`}
                    data-wire-target-kind="trigger_start"
                    onPointerDown={(event) => event.stopPropagation()}
                    onClick={(event) => {
                      event.stopPropagation();
                      onRerouteStep({ kind: "trigger_start" });
                    }}
                  >
                    insert first
                  </button>
                ) : null}

                {node.id !== TRIGGER_NODE_ID ? (
                  <>
                    <button
                      type="button"
                      className={`absolute -left-9 top-2 rounded-md border bg-white px-2 py-1 text-[10px] font-semibold uppercase tracking-wide ${
                        isHoveredRerouteTarget({ kind: "before", targetId: node.id })
                          ? "border-sky-500 text-sky-800"
                          : "border-zinc-300 text-zinc-700"
                      }`}
                      data-wire-target-kind="before"
                      data-wire-target-id={node.id}
                      onPointerDown={(event) => event.stopPropagation()}
                      onClick={(event) => {
                        event.stopPropagation();
                        onRerouteStep({ kind: "before", targetId: node.id });
                      }}
                    >
                      before
                    </button>
                    <button
                      type="button"
                      className={`absolute -left-9 bottom-2 rounded-md border bg-white px-2 py-1 text-[10px] font-semibold uppercase tracking-wide ${
                        isHoveredRerouteTarget({ kind: "after", targetId: node.id })
                          ? "border-sky-500 text-sky-800"
                          : "border-zinc-300 text-zinc-700"
                      }`}
                      data-wire-target-kind="after"
                      data-wire-target-id={node.id}
                      onPointerDown={(event) => event.stopPropagation()}
                      onClick={(event) => {
                        event.stopPropagation();
                        onRerouteStep({ kind: "after", targetId: node.id });
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
                      className={`absolute -bottom-8 left-2 rounded-md border bg-white px-2 py-1 text-[10px] font-semibold uppercase tracking-wide ${
                        isHoveredRerouteTarget({ kind: "then", targetId: node.id })
                          ? "border-sky-500 text-sky-800"
                          : "border-emerald-300 text-emerald-700"
                      }`}
                      data-wire-target-kind="then"
                      data-wire-target-id={node.id}
                      onPointerDown={(event) => event.stopPropagation()}
                      onClick={(event) => {
                        event.stopPropagation();
                        onRerouteStep({ kind: "then", targetId: node.id });
                      }}
                    >
                      to {stepForNode.thenLabel || "yes"}
                    </button>
                    <button
                      type="button"
                      className={`absolute -bottom-8 right-2 rounded-md border bg-white px-2 py-1 text-[10px] font-semibold uppercase tracking-wide ${
                        isHoveredRerouteTarget({ kind: "else", targetId: node.id })
                          ? "border-sky-500 text-sky-800"
                          : "border-zinc-300 text-zinc-700"
                      }`}
                      data-wire-target-kind="else"
                      data-wire-target-id={node.id}
                      onPointerDown={(event) => event.stopPropagation()}
                      onClick={(event) => {
                        event.stopPropagation();
                        onRerouteStep({ kind: "else", targetId: node.id });
                      }}
                    >
                      to {stepForNode.elseLabel || "no"}
                    </button>
                  </>
                ) : null}
              </>
            ) : null}
          </CanvasNode>
        );
      })}
    </WorkflowCanvas>
  );
}

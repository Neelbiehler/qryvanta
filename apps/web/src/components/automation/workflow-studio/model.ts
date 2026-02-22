import {
  type WorkflowConditionOperatorDto,
  type WorkflowStepDto,
} from "@/lib/api";

export type TriggerType = "manual" | "runtime_record_created";
export type ActionType = "log_message" | "create_runtime_record";
export type InspectorNode = "trigger" | "step";

export type DraftLogStep = {
  id: string;
  type: "log_message";
  message: string;
};

export type DraftCreateStep = {
  id: string;
  type: "create_runtime_record";
  entityLogicalName: string;
  dataJson: string;
};

export type DraftConditionStep = {
  id: string;
  type: "condition";
  fieldPath: string;
  operator: WorkflowConditionOperatorDto;
  valueJson: string;
  thenLabel: string;
  elseLabel: string;
  thenSteps: DraftWorkflowStep[];
  elseSteps: DraftWorkflowStep[];
};

export type DraftWorkflowStep =
  | DraftLogStep
  | DraftCreateStep
  | DraftConditionStep;

export type CanvasNodeDescriptor = {
  id: string;
  title: string;
  subtitle: string;
  kind: "trigger" | "step";
  tone: "trigger" | "action" | "condition";
};

export type CanvasEdgeDescriptor = {
  id: string;
  from: string;
  to: string;
  label?: string;
};

export type CanvasPosition = {
  x: number;
  y: number;
};

export type RerouteTarget =
  | { kind: "trigger_start" }
  | { kind: "before" | "after"; targetId: string }
  | { kind: "then" | "else"; targetId: string };

export type CanvasHistorySnapshot = {
  triggerType: TriggerType;
  triggerEntityLogicalName: string;
  steps: DraftWorkflowStep[];
  nodePositions: Record<string, CanvasPosition>;
  selectedStepId: string | null;
  inspectorNode: InspectorNode;
};

export type SelectionBoxState = {
  startX: number;
  startY: number;
  currentX: number;
  currentY: number;
};

export type CatalogInsertMode =
  | "after_selected"
  | "root"
  | "then_selected"
  | "else_selected";

export const CONDITION_OPERATORS: WorkflowConditionOperatorDto[] = [
  "equals",
  "not_equals",
  "exists",
];

export const TRIGGER_OPTIONS: Array<{
  value: TriggerType;
  label: string;
}> = [
  { value: "manual", label: "Manual trigger" },
  { value: "runtime_record_created", label: "Record created" },
];

export const STEP_LIBRARY: Array<{
  type: DraftWorkflowStep["type"];
  label: string;
  description: string;
}> = [
  {
    type: "log_message",
    label: "Log message",
    description: "Add a diagnostics log step.",
  },
  {
    type: "create_runtime_record",
    label: "Create record",
    description: "Create a new runtime record.",
  },
  {
    type: "condition",
    label: "Condition",
    description: "Branch into Yes/No step paths.",
  },
];

export type FlowTemplateId =
  | "manual_trigger"
  | "webhook_trigger"
  | "log_info"
  | "log_warning"
  | "create_task"
  | "create_note"
  | "http_request"
  | "transform_payload"
  | "delay_step"
  | "condition_equals"
  | "condition_exists";

export type FlowTemplateCategory =
  | "trigger"
  | "logic"
  | "integration"
  | "data"
  | "operations";

export const FLOW_TEMPLATES: Array<{
  id: FlowTemplateId;
  label: string;
  description: string;
  category: FlowTemplateCategory;
  keywords: string[];
  target: "step" | "trigger";
}> = [
  {
    id: "manual_trigger",
    label: "Manual Trigger",
    description: "Starts this flow from manual run in the canvas toolbar.",
    category: "trigger",
    keywords: ["trigger", "manual", "start"],
    target: "trigger",
  },
  {
    id: "webhook_trigger",
    label: "Webhook Event Trigger",
    description: "Starts when a webhook_event runtime record is created.",
    category: "trigger",
    keywords: ["trigger", "webhook", "event", "start"],
    target: "trigger",
  },
  {
    id: "condition_equals",
    label: "If Equals",
    description: "Branch execution when a payload field equals a value.",
    category: "logic",
    keywords: ["if", "branch", "equals", "condition"],
    target: "step",
  },
  {
    id: "condition_exists",
    label: "If Exists",
    description: "Branch execution when a payload field exists.",
    category: "logic",
    keywords: ["if", "exists", "condition", "branch"],
    target: "step",
  },
  {
    id: "http_request",
    label: "HTTP Request",
    description: "Send an outbound HTTP call (modeled as integration log step).",
    category: "integration",
    keywords: ["http", "request", "api", "integration"],
    target: "step",
  },
  {
    id: "transform_payload",
    label: "Transform Payload",
    description: "Map input values into a structured integration payload record.",
    category: "integration",
    keywords: ["transform", "map", "payload", "integration"],
    target: "step",
  },
  {
    id: "delay_step",
    label: "Delay",
    description: "Insert a wait/delay semantic step for downstream processing.",
    category: "integration",
    keywords: ["delay", "wait", "timer"],
    target: "step",
  },
  {
    id: "create_task",
    label: "Create Task Record",
    description: "Create a task runtime record with follow-up defaults.",
    category: "data",
    keywords: ["create", "record", "task", "data"],
    target: "step",
  },
  {
    id: "create_note",
    label: "Create Note Record",
    description: "Create a note runtime record for activity capture.",
    category: "data",
    keywords: ["create", "record", "note", "data"],
    target: "step",
  },
  {
    id: "log_info",
    label: "Log Info",
    description: "Write an informational trace message.",
    category: "operations",
    keywords: ["log", "message", "trace", "ops"],
    target: "step",
  },
  {
    id: "log_warning",
    label: "Log Warning",
    description: "Write a warning trace message.",
    category: "operations",
    keywords: ["log", "warning", "message", "ops"],
    target: "step",
  },
];

export const TRIGGER_NODE_ID = "flow_trigger_node";
export const CANVAS_NODE_WIDTH = 230;
export const CANVAS_NODE_HEIGHT = 88;
export const CANVAS_PADDING = 12;
export const GRID_SIZE = 16;
export const LANE_WIDTH = 250;

export function rerouteTargetsEqual(
  left: RerouteTarget,
  right: RerouteTarget,
): boolean {
  if (left.kind !== right.kind) {
    return false;
  }

  if (left.kind === "trigger_start" && right.kind === "trigger_start") {
    return true;
  }

  if (left.kind === "trigger_start" || right.kind === "trigger_start") {
    return false;
  }

  return left.targetId === right.targetId;
}

export function rerouteTargetFromDataset(
  kind: string | undefined,
  targetId: string | undefined,
): RerouteTarget | null {
  if (kind === "trigger_start") {
    return { kind: "trigger_start" };
  }

  if (!targetId) {
    return null;
  }

  if (
    kind === "before" ||
    kind === "after" ||
    kind === "then" ||
    kind === "else"
  ) {
    return { kind, targetId };
  }

  return null;
}

export function parseJsonObject(
  value: string,
  fieldLabel: string,
): Record<string, unknown> {
  const parsed = JSON.parse(value) as unknown;
  if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
    throw new Error(`${fieldLabel} must be a JSON object.`);
  }

  return parsed as Record<string, unknown>;
}

export function parseJsonValue(value: string, fieldLabel: string): unknown {
  try {
    return JSON.parse(value) as unknown;
  } catch {
    throw new Error(`${fieldLabel} must be valid JSON.`);
  }
}

export function isTypingElement(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) {
    return false;
  }

  if (target.isContentEditable) {
    return true;
  }

  const tagName = target.tagName.toLowerCase();
  return tagName === "input" || tagName === "textarea" || tagName === "select";
}

export function cloneWorkflowSteps(
  steps: DraftWorkflowStep[],
): DraftWorkflowStep[] {
  return JSON.parse(JSON.stringify(steps)) as DraftWorkflowStep[];
}

export function cloneCanvasPositions(
  positions: Record<string, CanvasPosition>,
): Record<string, CanvasPosition> {
  return JSON.parse(JSON.stringify(positions)) as Record<string, CanvasPosition>;
}

export function summarizeStep(step: DraftWorkflowStep): string {
  switch (step.type) {
    case "log_message":
      return step.message.trim().length > 0
        ? `Log: ${step.message}`
        : "Log message";
    case "create_runtime_record":
      return step.entityLogicalName.trim().length > 0
        ? `Create: ${step.entityLogicalName}`
        : "Create runtime record";
    case "condition":
      return `${step.fieldPath || "[field path]"} ${step.operator}`;
    default:
      return "Step";
  }
}

function stepTitle(step: DraftWorkflowStep): string {
  if (step.type === "log_message") {
    return "Log message";
  }

  if (step.type === "create_runtime_record") {
    return "Create record";
  }

  return "Condition";
}

function stepTone(step: DraftWorkflowStep): CanvasNodeDescriptor["tone"] {
  if (step.type === "condition") {
    return "condition";
  }

  return "action";
}

function appendCanvasBranch(
  steps: DraftWorkflowStep[],
  parentId: string,
  nodes: CanvasNodeDescriptor[],
  edges: CanvasEdgeDescriptor[],
  branchLabel?: string,
) {
  let previousId: string | null = null;

  for (const step of steps) {
    nodes.push({
      id: step.id,
      title: stepTitle(step),
      subtitle: summarizeStep(step),
      kind: "step",
      tone: stepTone(step),
    });

    const from = previousId ?? parentId;
    edges.push({
      id: `${from}_${step.id}_${branchLabel ?? ""}`,
      from,
      to: step.id,
      label: previousId ? undefined : branchLabel,
    });

    if (step.type === "condition") {
      appendCanvasBranch(step.thenSteps, step.id, nodes, edges, step.thenLabel || "yes");
      appendCanvasBranch(step.elseSteps, step.id, nodes, edges, step.elseLabel || "no");
    }

    previousId = step.id;
  }
}

export function buildCanvasGraph(
  triggerSummary: string,
  steps: DraftWorkflowStep[],
): { nodes: CanvasNodeDescriptor[]; edges: CanvasEdgeDescriptor[] } {
  const nodes: CanvasNodeDescriptor[] = [
    {
      id: TRIGGER_NODE_ID,
      title: "Trigger",
      subtitle: triggerSummary,
      kind: "trigger",
      tone: "trigger",
    },
  ];
  const edges: CanvasEdgeDescriptor[] = [];

  appendCanvasBranch(steps, TRIGGER_NODE_ID, nodes, edges);
  return { nodes, edges };
}

export function buildDefaultCanvasPositions(
  steps: DraftWorkflowStep[],
): Record<string, CanvasPosition> {
  const positions: Record<string, CanvasPosition> = {
    [TRIGGER_NODE_ID]: { x: CANVAS_PADDING + 24, y: 280 },
  };

  let laneRow = 0;

  function allocateRow(preferredY?: number): number {
    const candidateRow =
      typeof preferredY === "number" ? Math.floor(Math.max(48, preferredY) / 112) : laneRow;
    laneRow = Math.max(laneRow + 1, candidateRow + 1);
    return candidateRow;
  }

  function rowToY(row: number): number {
    return 64 + row * 112;
  }

  function placeBranch(
    branchSteps: DraftWorkflowStep[],
    depth: number,
    preferredY?: number,
  ) {
    let localY = preferredY;

    for (const step of branchSteps) {
      const row = allocateRow(localY);
      const y = rowToY(row);
      positions[step.id] = {
        x: 280 + depth * LANE_WIDTH,
        y,
      };

      if (step.type === "condition") {
        placeBranch(step.thenSteps, depth + 1, y - 72);
        placeBranch(step.elseSteps, depth + 1, y + 72);
      }

      localY = y + 112;
    }
  }

  placeBranch(steps, 0);
  return positions;
}

export function maxCanvasDepth(steps: DraftWorkflowStep[]): number {
  let maxDepth = 0;

  function visit(branch: DraftWorkflowStep[], depth: number) {
    maxDepth = Math.max(maxDepth, depth);

    for (const step of branch) {
      if (step.type === "condition") {
        visit(step.thenSteps, depth + 1);
        visit(step.elseSteps, depth + 1);
      }
    }
  }

  visit(steps, 1);
  return maxDepth;
}

export function findStepById(
  steps: DraftWorkflowStep[],
  stepId: string,
): DraftWorkflowStep | null {
  for (const step of steps) {
    if (step.id === stepId) {
      return step;
    }

    if (step.type === "condition") {
      const fromThen = findStepById(step.thenSteps, stepId);
      if (fromThen) {
        return fromThen;
      }

      const fromElse = findStepById(step.elseSteps, stepId);
      if (fromElse) {
        return fromElse;
      }
    }
  }

  return null;
}

export function updateStepById(
  steps: DraftWorkflowStep[],
  stepId: string,
  updater: (step: DraftWorkflowStep) => DraftWorkflowStep,
): DraftWorkflowStep[] {
  return steps.map((step) => {
    if (step.id === stepId) {
      return updater(step);
    }

    if (step.type !== "condition") {
      return step;
    }

    return {
      ...step,
      thenSteps: updateStepById(step.thenSteps, stepId, updater),
      elseSteps: updateStepById(step.elseSteps, stepId, updater),
    };
  });
}

export function removeStepById(
  steps: DraftWorkflowStep[],
  stepId: string,
): DraftWorkflowStep[] {
  return steps
    .filter((step) => step.id !== stepId)
    .map((step) => {
      if (step.type !== "condition") {
        return step;
      }

      return {
        ...step,
        thenSteps: removeStepById(step.thenSteps, stepId),
        elseSteps: removeStepById(step.elseSteps, stepId),
      };
    });
}

export function extractStepById(
  steps: DraftWorkflowStep[],
  stepId: string,
): { steps: DraftWorkflowStep[]; extracted: DraftWorkflowStep | null } {
  let extracted: DraftWorkflowStep | null = null;

  const nextSteps = steps
    .map((step) => {
      if (step.id === stepId) {
        extracted = step;
        return null;
      }

      if (step.type !== "condition") {
        return step;
      }

      const thenResult = extractStepById(step.thenSteps, stepId);
      const elseResult = extractStepById(step.elseSteps, stepId);

      if (thenResult.extracted) {
        extracted = thenResult.extracted;
      }

      if (elseResult.extracted) {
        extracted = elseResult.extracted;
      }

      return {
        ...step,
        thenSteps: thenResult.steps,
        elseSteps: elseResult.steps,
      };
    })
    .filter((step): step is DraftWorkflowStep => step !== null);

  return { steps: nextSteps, extracted };
}

export function insertStepRelativeToTarget(
  steps: DraftWorkflowStep[],
  targetId: string,
  mode: "before" | "after" | "then" | "else",
  stepToInsert: DraftWorkflowStep,
): { steps: DraftWorkflowStep[]; inserted: boolean } {
  let inserted = false;
  const nextSteps: DraftWorkflowStep[] = [];

  for (const step of steps) {
    if (step.id === targetId) {
      if (mode === "before") {
        nextSteps.push(stepToInsert);
        nextSteps.push(step);
        inserted = true;
        continue;
      }

      if (mode === "after") {
        nextSteps.push(step);
        nextSteps.push(stepToInsert);
        inserted = true;
        continue;
      }

      if (step.type === "condition") {
        nextSteps.push({
          ...step,
          thenSteps: mode === "then" ? [...step.thenSteps, stepToInsert] : step.thenSteps,
          elseSteps: mode === "else" ? [...step.elseSteps, stepToInsert] : step.elseSteps,
        });
        inserted = true;
        continue;
      }
    }

    if (step.type !== "condition") {
      nextSteps.push(step);
      continue;
    }

    const thenResult = insertStepRelativeToTarget(
      step.thenSteps,
      targetId,
      mode,
      stepToInsert,
    );
    const elseResult = thenResult.inserted
      ? { steps: step.elseSteps, inserted: false }
      : insertStepRelativeToTarget(step.elseSteps, targetId, mode, stepToInsert);

    if (thenResult.inserted || elseResult.inserted) {
      inserted = true;
    }

    nextSteps.push({
      ...step,
      thenSteps: thenResult.steps,
      elseSteps: elseResult.steps,
    });
  }

  return { steps: nextSteps, inserted };
}

export function stepContainsId(
  step: DraftWorkflowStep,
  stepId: string,
): boolean {
  if (step.id === stepId) {
    return true;
  }

  if (step.type !== "condition") {
    return false;
  }

  return (
    step.thenSteps.some((nestedStep) => stepContainsId(nestedStep, stepId)) ||
    step.elseSteps.some((nestedStep) => stepContainsId(nestedStep, stepId))
  );
}

export function appendStepToBranch(
  steps: DraftWorkflowStep[],
  conditionStepId: string,
  branch: "then" | "else",
  draftStep: DraftWorkflowStep,
): DraftWorkflowStep[] {
  return steps.map((step) => {
    if (step.id === conditionStepId && step.type === "condition") {
      if (branch === "then") {
        return { ...step, thenSteps: [...step.thenSteps, draftStep] };
      }

      return { ...step, elseSteps: [...step.elseSteps, draftStep] };
    }

    if (step.type !== "condition") {
      return step;
    }

    return {
      ...step,
      thenSteps: appendStepToBranch(step.thenSteps, conditionStepId, branch, draftStep),
      elseSteps: appendStepToBranch(step.elseSteps, conditionStepId, branch, draftStep),
    };
  });
}

export function firstActionFromSteps(
  steps: WorkflowStepDto[],
): {
  actionType: ActionType;
  actionEntityLogicalName: string | null;
  actionPayload: Record<string, unknown>;
} | null {
  for (const step of steps) {
    if (step.type === "log_message") {
      return {
        actionType: "log_message",
        actionEntityLogicalName: null,
        actionPayload: { message: step.message },
      };
    }

    if (step.type === "create_runtime_record") {
      return {
        actionType: "create_runtime_record",
        actionEntityLogicalName: step.entity_logical_name,
        actionPayload: step.data,
      };
    }

    const fromThen = firstActionFromSteps(step.then_steps);
    if (fromThen) {
      return fromThen;
    }

    const fromElse = firstActionFromSteps(step.else_steps);
    if (fromElse) {
      return fromElse;
    }
  }

  return null;
}

export function createDraftFromTransport(
  step: WorkflowStepDto,
  createId: () => string,
): DraftWorkflowStep {
  if (step.type === "log_message") {
    return {
      id: createId(),
      type: "log_message",
      message: step.message,
    };
  }

  if (step.type === "create_runtime_record") {
    return {
      id: createId(),
      type: "create_runtime_record",
      entityLogicalName: step.entity_logical_name,
      dataJson: JSON.stringify(step.data, null, 2),
    };
  }

  return {
    id: createId(),
    type: "condition",
    fieldPath: step.field_path,
    operator: step.operator,
    valueJson: step.operator === "exists" ? "null" : JSON.stringify(step.value ?? "", null, 2),
    thenLabel: step.then_label ?? "Yes",
    elseLabel: step.else_label ?? "No",
    thenSteps: step.then_steps.map((nestedStep) => createDraftFromTransport(nestedStep, createId)),
    elseSteps: step.else_steps.map((nestedStep) => createDraftFromTransport(nestedStep, createId)),
  };
}

export function describeTrigger(
  triggerType: TriggerType,
  triggerEntityLogicalName: string,
): string {
  if (triggerType === "manual") {
    return "Manual trigger from the automation console.";
  }

  if (triggerEntityLogicalName.trim().length === 0) {
    return "Runtime record created in [entity required].";
  }

  return `Runtime record created in '${triggerEntityLogicalName}'.`;
}

export function createDraftStep(
  stepType: DraftWorkflowStep["type"],
  createId: () => string,
): DraftWorkflowStep {
  if (stepType === "log_message") {
    return {
      id: createId(),
      type: "log_message",
      message: "workflow fired",
    };
  }

  if (stepType === "create_runtime_record") {
    return {
      id: createId(),
      type: "create_runtime_record",
      entityLogicalName: "task",
      dataJson: JSON.stringify({ title: "Follow-up" }, null, 2),
    };
  }

  return {
    id: createId(),
    type: "condition",
    fieldPath: "status",
    operator: "equals",
    valueJson: JSON.stringify("open"),
    thenLabel: "Yes",
    elseLabel: "No",
    thenSteps: [
      {
        id: createId(),
        type: "log_message",
        message: "matched condition",
      },
    ],
    elseSteps: [
      {
        id: createId(),
        type: "log_message",
        message: "did not match condition",
      },
    ],
  };
}

export function createTemplateStep(
  templateId: FlowTemplateId,
  createId: () => string,
): DraftWorkflowStep {
  switch (templateId) {
    case "manual_trigger":
    case "webhook_trigger":
      return {
        id: createId(),
        type: "log_message",
        message: "trigger template applied",
      };
    case "log_info":
      return {
        id: createId(),
        type: "log_message",
        message: "[INFO] flow step executed",
      };
    case "log_warning":
      return {
        id: createId(),
        type: "log_message",
        message: "[WARN] requires attention",
      };
    case "create_task":
      return {
        id: createId(),
        type: "create_runtime_record",
        entityLogicalName: "task",
        dataJson: JSON.stringify({ title: "Follow-up", priority: "normal" }, null, 2),
      };
    case "create_note":
      return {
        id: createId(),
        type: "create_runtime_record",
        entityLogicalName: "note",
        dataJson: JSON.stringify({ title: "Activity Note", body: "auto generated" }, null, 2),
      };
    case "http_request":
      return {
        id: createId(),
        type: "log_message",
        message: "[HTTP] GET https://api.example.com/resource",
      };
    case "transform_payload":
      return {
        id: createId(),
        type: "create_runtime_record",
        entityLogicalName: "integration_payload",
        dataJson: JSON.stringify(
          {
            source: "trigger",
            transformed: true,
            mapping_version: "v1",
          },
          null,
          2,
        ),
      };
    case "delay_step":
      return {
        id: createId(),
        type: "log_message",
        message: "[DELAY] wait 5m",
      };
    case "condition_exists":
      return {
        id: createId(),
        type: "condition",
        fieldPath: "contact.email",
        operator: "exists",
        valueJson: "null",
        thenLabel: "Found",
        elseLabel: "Missing",
        thenSteps: [
          {
            id: createId(),
            type: "log_message",
            message: "email found",
          },
        ],
        elseSteps: [
          {
            id: createId(),
            type: "create_runtime_record",
            entityLogicalName: "task",
            dataJson: JSON.stringify({ title: "Collect missing email" }, null, 2),
          },
        ],
      };
    case "condition_equals":
    default:
      return {
        id: createId(),
        type: "condition",
        fieldPath: "status",
        operator: "equals",
        valueJson: JSON.stringify("open"),
        thenLabel: "Open",
        elseLabel: "Closed",
        thenSteps: [
          {
            id: createId(),
            type: "log_message",
            message: "status is open",
          },
        ],
        elseSteps: [
          {
            id: createId(),
            type: "log_message",
            message: "status is not open",
          },
        ],
      };
  }
}

export function resolveTemplateList(
  query: string,
  category: "all" | FlowTemplateCategory,
) {
  const withScore = FLOW_TEMPLATES.map((template) => {
    if (category !== "all" && template.category !== category) {
      return null;
    }

    const haystacks = [template.label, template.description, ...template.keywords].map(
      (value) => value.toLowerCase(),
    );

    if (!query) {
      return { template, score: 0 };
    }

    let score = 0;
    for (const haystack of haystacks) {
      if (haystack === query) {
        score += 120;
      } else if (haystack.startsWith(query)) {
        score += 90;
      } else if (haystack.includes(query)) {
        score += 45;
      } else {
        const charsMatched = query.split("").every((char) => haystack.includes(char));
        if (charsMatched) {
          score += 15;
        }
      }
    }

    if (score === 0) {
      return null;
    }

    return { template, score };
  })
    .filter((entry): entry is { template: (typeof FLOW_TEMPLATES)[number]; score: number } => Boolean(entry))
    .sort((left, right) => right.score - left.score || left.template.label.localeCompare(right.template.label));

  return withScore.map((entry) => entry.template);
}

import {
  type WorkflowConditionOperatorDto,
  type WorkflowRunStepTraceResponse,
  type WorkflowStepDto,
} from "@/lib/api";

export {
  createDraftStep,
  createTemplateStep,
  FLOW_TEMPLATES,
  resolveTemplateList,
  STEP_LIBRARY,
  triggerTemplateConfig,
} from "@/components/automation/workflow-studio/model/flow-templates";
export type {
  FlowTemplateCategory,
  FlowTemplateId,
} from "@/components/automation/workflow-studio/model/flow-templates";

export type TriggerType =
  | "manual"
  | "runtime_record_created"
  | "runtime_record_updated"
  | "runtime_record_deleted"
  | "schedule_tick";
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

export type CanvasHistorySnapshot = {
  triggerType: TriggerType;
  triggerEntityLogicalName: string;
  steps: DraftWorkflowStep[];
  selectedStepId: string | null;
  inspectorNode: InspectorNode;
};

export type WorkflowValidationIssue = {
  id: string;
  stepId: string | null;
  level: "error" | "warning";
  message: string;
};

export type StepPathIndex = {
  byStepId: Record<string, string>;
  byPath: Record<string, DraftWorkflowStep>;
};

export type DynamicTokenOption = {
  token: string;
  label: string;
  source: "trigger" | "step" | "runtime";
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
  { value: "runtime_record_updated", label: "Record updated" },
  { value: "runtime_record_deleted", label: "Record deleted" },
  { value: "schedule_tick", label: "Schedule tick" },
];

export const RUNTIME_TRIGGER_ENTITY_PRESETS: Array<{
  value: string;
  label: string;
}> = [
  { value: "contact", label: "Contact created" },
  { value: "webhook_event", label: "Webhook received" },
  { value: "form_submission", label: "Form submitted" },
  { value: "inbound_email", label: "Inbound email" },
  { value: "approval_request", label: "Approval requested" },
  { value: "schedule_hourly", label: "Hourly schedule tick" },
  { value: "schedule_daily", label: "Daily schedule tick" },
];

export const SCHEDULE_TRIGGER_KEY_PRESETS: Array<{
  value: string;
  label: string;
}> = [
  { value: "hourly", label: "Hourly" },
  { value: "daily", label: "Daily" },
  { value: "daily_utc_0900", label: "Daily 09:00 UTC" },
  { value: "weekday_utc_0900", label: "Weekday 09:00 UTC" },
];



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

export function buildStepPathIndex(steps: DraftWorkflowStep[]): StepPathIndex {
  const byStepId: Record<string, string> = {};
  const byPath: Record<string, DraftWorkflowStep> = {};

  function visit(branchSteps: DraftWorkflowStep[], prefix: string) {
    branchSteps.forEach((step, index) => {
      const stepPath = prefix.length === 0 ? `${index}` : `${prefix}.${index}`;
      byStepId[step.id] = stepPath;
      byPath[stepPath] = step;

      if (step.type === "condition") {
        visit(step.thenSteps, `${stepPath}.then`);
        visit(step.elseSteps, `${stepPath}.else`);
      }
    });
  }

  visit(steps, "");
  return { byStepId, byPath };
}

export function stepTraceMapByPath(
  stepTraces: WorkflowRunStepTraceResponse[] | undefined,
): Record<string, WorkflowRunStepTraceResponse> {
  if (!stepTraces || stepTraces.length === 0) {
    return {};
  }

  return stepTraces.reduce(
    (map, trace) => {
      map[trace.step_path] = trace;
      return map;
    },
    {} as Record<string, WorkflowRunStepTraceResponse>,
  );
}

function orderedSteps(steps: DraftWorkflowStep[]): DraftWorkflowStep[] {
  const flattened: DraftWorkflowStep[] = [];

  function visit(branchSteps: DraftWorkflowStep[]) {
    for (const step of branchSteps) {
      flattened.push(step);
      if (step.type === "condition") {
        visit(step.thenSteps);
        visit(step.elseSteps);
      }
    }
  }

  visit(steps);
  return flattened;
}

function stepTokenLabel(step: DraftWorkflowStep): string {
  if (step.type === "log_message") {
    return `Log step (${step.id})`;
  }

  if (step.type === "create_runtime_record") {
    return `Create record (${step.entityLogicalName || step.id})`;
  }

  return `Condition (${step.id})`;
}

export function dynamicTokensForStep(
  steps: DraftWorkflowStep[],
  selectedStepId: string | null,
  triggerPayloadFieldPaths: string[] = [],
): DynamicTokenOption[] {
  const baseTokens: DynamicTokenOption[] = [
    { token: "{{trigger.type}}", label: "Trigger type", source: "trigger" },
    { token: "{{trigger.entity}}", label: "Trigger entity", source: "trigger" },
    { token: "{{trigger.payload.id}}", label: "Trigger payload id", source: "trigger" },
    {
      token: "{{trigger.payload.status}}",
      label: "Trigger payload status",
      source: "trigger",
    },
    { token: "{{run.id}}", label: "Run id", source: "runtime" },
    { token: "{{run.attempt}}", label: "Run attempt", source: "runtime" },
    { token: "{{now.iso}}", label: "Current time (ISO)", source: "runtime" },
    ...triggerPayloadFieldPaths.map((fieldPath) => ({
      token: `{{trigger.payload.${fieldPath}}}`,
      label: `Trigger payload ${fieldPath}`,
      source: "trigger" as const,
    })),
  ];

  const dedupedBaseTokens = Array.from(
    new Map(baseTokens.map((token) => [token.token, token])).values(),
  );

  if (!selectedStepId) {
    return dedupedBaseTokens;
  }

  const flattened = orderedSteps(steps);
  const selectedIndex = flattened.findIndex((step) => step.id === selectedStepId);
  if (selectedIndex <= 0) {
    return dedupedBaseTokens;
  }

  const previousSteps = flattened.slice(0, selectedIndex);
  const previousStepTokens = previousSteps.map((step) => ({
    token: `{{steps.${step.id}.output}}`,
    label: `${stepTokenLabel(step)} output`,
    source: "step" as const,
  }));

  return [...dedupedBaseTokens, ...previousStepTokens];
}

export function duplicateStepWithNewIds(
  step: DraftWorkflowStep,
  createId: () => string,
): DraftWorkflowStep {
  if (step.type === "log_message") {
    return {
      ...step,
      id: createId(),
    };
  }

  if (step.type === "create_runtime_record") {
    return {
      ...step,
      id: createId(),
    };
  }

  return {
    ...step,
    id: createId(),
    thenSteps: step.thenSteps.map((nestedStep) =>
      duplicateStepWithNewIds(nestedStep, createId),
    ),
    elseSteps: step.elseSteps.map((nestedStep) =>
      duplicateStepWithNewIds(nestedStep, createId),
    ),
  };
}

export function duplicateStepById(
  steps: DraftWorkflowStep[],
  stepId: string,
  createId: () => string,
): { steps: DraftWorkflowStep[]; duplicatedStepId: string | null } {
  let duplicatedStepId: string | null = null;

  const nextSteps: DraftWorkflowStep[] = [];
  for (const step of steps) {
    if (step.id === stepId && duplicatedStepId === null) {
      const duplicate = duplicateStepWithNewIds(step, createId);
      duplicatedStepId = duplicate.id;
      nextSteps.push(step, duplicate);
      continue;
    }

    if (step.type !== "condition") {
      nextSteps.push(step);
      continue;
    }

    const thenResult = duplicateStepById(step.thenSteps, stepId, createId);
    const elseResult =
      thenResult.duplicatedStepId === null
        ? duplicateStepById(step.elseSteps, stepId, createId)
        : { steps: step.elseSteps, duplicatedStepId: null };

    if (thenResult.duplicatedStepId) {
      duplicatedStepId = thenResult.duplicatedStepId;
    }

    if (elseResult.duplicatedStepId) {
      duplicatedStepId = elseResult.duplicatedStepId;
    }

    nextSteps.push({
      ...step,
      thenSteps: thenResult.steps,
      elseSteps: elseResult.steps,
    });
  }

  return { steps: nextSteps, duplicatedStepId };
}

export function collectWorkflowValidationIssues(
  triggerType: TriggerType,
  triggerEntityLogicalName: string,
  steps: DraftWorkflowStep[],
): WorkflowValidationIssue[] {
  const issues: WorkflowValidationIssue[] = [];
  let counter = 0;

  function addIssue(
    issue: Omit<WorkflowValidationIssue, "id">,
  ) {
    counter += 1;
    issues.push({ ...issue, id: `workflow_issue_${counter}` });
  }

  function validateBranch(branchSteps: DraftWorkflowStep[]) {
    for (const step of branchSteps) {
      if (step.type === "log_message") {
        if (step.message.trim().length === 0) {
          addIssue({
            stepId: step.id,
            level: "error",
            message: "Log message step is empty.",
          });
        }
        continue;
      }

      if (step.type === "create_runtime_record") {
        if (step.entityLogicalName.trim().length === 0) {
          addIssue({
            stepId: step.id,
            level: "error",
            message: "Create record step is missing an entity logical name.",
          });
        }

        try {
          const parsed = JSON.parse(step.dataJson) as unknown;
          const isObject = parsed && typeof parsed === "object" && !Array.isArray(parsed);
          if (!isObject) {
            addIssue({
              stepId: step.id,
              level: "error",
              message: "Create record step data must be a JSON object.",
            });
          }
        } catch {
          addIssue({
            stepId: step.id,
            level: "error",
            message: "Create record step data contains invalid JSON.",
          });
        }
        continue;
      }

      if (step.fieldPath.trim().length === 0) {
        addIssue({
          stepId: step.id,
          level: "error",
          message: "Condition step requires a payload field path.",
        });
      }

      if (step.operator !== "exists") {
        try {
          JSON.parse(step.valueJson);
        } catch {
          addIssue({
            stepId: step.id,
            level: "error",
            message: "Condition value must be valid JSON for non-exists operators.",
          });
        }
      }

      if (step.thenSteps.length === 0 && step.elseSteps.length === 0) {
        addIssue({
          stepId: step.id,
          level: "error",
          message: "Condition step must include at least one action in a branch.",
        });
      }

      validateBranch(step.thenSteps);
      validateBranch(step.elseSteps);
    }
  }

  if (steps.length === 0) {
    addIssue({
      stepId: null,
      level: "error",
      message: "Flow canvas requires at least one step.",
    });
  }

  if (
    triggerType !== "manual" &&
    triggerEntityLogicalName.trim().length === 0
  ) {
    addIssue({
      stepId: null,
      level: "error",
      message:
        triggerType === "schedule_tick"
          ? "Schedule tick trigger requires a schedule key."
          : "Runtime record trigger requires an entity logical name.",
    });
  }

  validateBranch(steps);
  return issues;
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

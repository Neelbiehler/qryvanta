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
  | "schedule_tick"
  | "webhook_received"
  | "form_submitted"
  | "inbound_email_received"
  | "approval_event_received";
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
  dataFields: DraftObjectField[];
};

export type DraftUpdateStep = {
  id: string;
  type: "update_runtime_record";
  entityLogicalName: string;
  recordId: string;
  dataFields: DraftObjectField[];
};

export type DraftDeleteStep = {
  id: string;
  type: "delete_runtime_record";
  entityLogicalName: string;
  recordId: string;
};

export type DraftSendEmailStep = {
  id: string;
  type: "send_email";
  to: string;
  subject: string;
  body: string;
  htmlBody: string;
};

export type DraftValueKind = "string" | "number" | "boolean" | "null" | "json";

export type DraftObjectField = {
  id: string;
  key: string;
  valueKind: DraftValueKind;
  value: string;
};

export type DraftArrayItem = {
  id: string;
  valueKind: DraftValueKind;
  value: string;
};

export type DraftHttpBodyMode = "none" | "object" | "array" | "scalar" | "json";

export type DraftHttpRequestStep = {
  id: string;
  type: "http_request";
  method: string;
  url: string;
  headersJson: string;
  headerSecretRefsJson: string;
  bodyMode: DraftHttpBodyMode;
  bodyFields: DraftObjectField[];
  bodyArrayItems: DraftArrayItem[];
  bodyScalarKind: DraftValueKind;
  bodyScalarValue: string;
  bodyJson: string;
};

export type DraftWebhookStep = {
  id: string;
  type: "webhook";
  endpoint: string;
  event: string;
  headersJson: string;
  headerSecretRefsJson: string;
  payloadFields: DraftObjectField[];
};

export type DraftAssignOwnerStep = {
  id: string;
  type: "assign_owner";
  entityLogicalName: string;
  recordId: string;
  ownerId: string;
  reason: string;
};

export type DraftApprovalRequestStep = {
  id: string;
  type: "approval_request";
  entityLogicalName: string;
  recordId: string;
  requestType: string;
  requestedBy: string;
  approverId: string;
  reason: string;
  payloadFields: DraftObjectField[];
};

export type DraftDelayStep = {
  id: string;
  type: "delay";
  durationMs: string;
  reason: string;
};

export type DraftConditionStep = {
  id: string;
  type: "condition";
  fieldPath: string;
  operator: WorkflowConditionOperatorDto;
  valueKind: DraftValueKind;
  valueText: string;
  thenLabel: string;
  elseLabel: string;
  thenSteps: DraftWorkflowStep[];
  elseSteps: DraftWorkflowStep[];
};

export type DraftWorkflowStep =
  | DraftLogStep
  | DraftCreateStep
  | DraftUpdateStep
  | DraftDeleteStep
  | DraftSendEmailStep
  | DraftHttpRequestStep
  | DraftWebhookStep
  | DraftAssignOwnerStep
  | DraftApprovalRequestStep
  | DraftDelayStep
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
  { value: "webhook_received", label: "Webhook received" },
  { value: "form_submitted", label: "Form submitted" },
  { value: "inbound_email_received", label: "Inbound email" },
  { value: "approval_event_received", label: "Approval event" },
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

export function parseJsonStringMap(
  value: string,
  fieldLabel: string,
): Record<string, string> {
  const parsed = parseJsonObject(value, fieldLabel);

  for (const [key, entry] of Object.entries(parsed)) {
    if (typeof entry !== "string") {
      throw new Error(`${fieldLabel} field "${key}" must be a string.`);
    }
  }

  return parsed as Record<string, string>;
}

export function parseJsonValue(value: string, fieldLabel: string): unknown {
  try {
    return JSON.parse(value) as unknown;
  } catch {
    throw new Error(`${fieldLabel} must be valid JSON.`);
  }
}

export function createDraftFieldId(): string {
  return `draft_field_${Math.random().toString(36).slice(2, 10)}`;
}

function inferDraftValueKind(value: unknown): DraftValueKind {
  if (value === null) {
    return "null";
  }
  if (typeof value === "string") {
    return "string";
  }
  if (typeof value === "number") {
    return "number";
  }
  if (typeof value === "boolean") {
    return "boolean";
  }
  return "json";
}

function stringifyDraftValue(value: unknown): string {
  if (value === null) {
    return "";
  }
  if (typeof value === "string") {
    return value;
  }
  if (typeof value === "number" || typeof value === "boolean") {
    return String(value);
  }
  return JSON.stringify(value, null, 2);
}

export function createDraftObjectField(
  key = "",
  value: unknown = "",
  id = createDraftFieldId(),
): DraftObjectField {
  return {
    id,
    key,
    valueKind: inferDraftValueKind(value),
    value: stringifyDraftValue(value),
  };
}

export function createDraftObjectFieldsFromValue(
  value: Record<string, unknown>,
): DraftObjectField[] {
  return Object.entries(value).map(([key, entry]) => createDraftObjectField(key, entry));
}

export function createDraftArrayItem(
  value: unknown = "",
  id = createDraftFieldId(),
): DraftArrayItem {
  return {
    id,
    valueKind: inferDraftValueKind(value),
    value: stringifyDraftValue(value),
  };
}

export function createDraftArrayItemsFromValue(value: unknown[]): DraftArrayItem[] {
  return value.map((entry) => createDraftArrayItem(entry));
}

export function parseDraftValue(
  valueKind: DraftValueKind,
  value: string,
  fieldLabel: string,
): unknown {
  if (valueKind === "string") {
    return value;
  }

  if (valueKind === "number") {
    const parsed = Number(value);
    if (!Number.isFinite(parsed)) {
      throw new Error(`${fieldLabel} must be a valid number.`);
    }
    return parsed;
  }

  if (valueKind === "boolean") {
    if (value !== "true" && value !== "false") {
      throw new Error(`${fieldLabel} must be true or false.`);
    }
    return value === "true";
  }

  if (valueKind === "null") {
    return null;
  }

  return parseJsonValue(value, fieldLabel);
}

export function parseDraftObjectFields(
  fields: DraftObjectField[],
  fieldLabel: string,
): Record<string, unknown> {
  return fields.reduce<Record<string, unknown>>((acc, field, index) => {
    const key = field.key.trim();
    if (key.length === 0) {
      return acc;
    }

    acc[key] = parseDraftValue(field.valueKind, field.value, `${fieldLabel} field "${key}"`);
    return acc;
  }, {});
}

export function parseDraftArrayItems(
  items: DraftArrayItem[],
  fieldLabel: string,
): unknown[] {
  return items.map((item, index) =>
    parseDraftValue(item.valueKind, item.value, `${fieldLabel} item ${index + 1}`),
  );
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
    case "update_runtime_record":
      return step.entityLogicalName.trim().length > 0
        ? `Update: ${step.entityLogicalName} ${step.recordId || ""}`.trim()
        : "Update record";
    case "delete_runtime_record":
      return step.entityLogicalName.trim().length > 0
        ? `Delete: ${step.entityLogicalName} ${step.recordId || ""}`.trim()
        : "Delete record";
    case "send_email":
      return step.subject.trim().length > 0
        ? `Email: ${step.subject}`
        : "Send email";
    case "http_request":
      return step.url.trim().length > 0 ? `HTTP: ${step.method} ${step.url}` : "HTTP request";
    case "webhook":
      return step.event.trim().length > 0
        ? `Webhook: ${step.event}`
        : "Webhook dispatch";
    case "assign_owner":
      return step.ownerId.trim().length > 0
        ? `Assign owner: ${step.ownerId}`
        : "Assign owner";
    case "approval_request":
      return step.requestType.trim().length > 0
        ? `Approval: ${step.requestType}`
        : "Approval request";
    case "delay":
      return step.durationMs.trim().length > 0
        ? `Delay: ${step.durationMs} ms`
        : "Delay";
    case "condition":
      return `${step.fieldPath || "[field path]"} ${step.operator}`;
    default:
      return "Step";
  }
}

function isJsonObjectValue(
  value: unknown,
): value is Record<string, unknown> {
  return Boolean(value) && typeof value === "object" && !Array.isArray(value);
}

function isJsonArrayValue(value: unknown): value is unknown[] {
  return Array.isArray(value);
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

  if (step.type === "update_runtime_record") {
    return `Update record (${step.entityLogicalName || step.id})`;
  }

  if (step.type === "delete_runtime_record") {
    return `Delete record (${step.entityLogicalName || step.id})`;
  }

  if (step.type === "send_email") {
    return `Send email (${step.subject || step.id})`;
  }

  if (step.type === "http_request") {
    return `HTTP request (${step.url || step.id})`;
  }

  if (step.type === "webhook") {
    return `Webhook (${step.event || step.id})`;
  }

  if (step.type === "assign_owner") {
    return `Assign owner (${step.ownerId || step.id})`;
  }

  if (step.type === "approval_request") {
    return `Approval request (${step.requestType || step.id})`;
  }

  if (step.type === "delay") {
    return `Delay (${step.durationMs || step.id})`;
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

  if (
    step.type === "update_runtime_record" ||
    step.type === "delete_runtime_record" ||
    step.type === "send_email" ||
    step.type === "http_request" ||
    step.type === "webhook" ||
    step.type === "assign_owner" ||
    step.type === "approval_request" ||
    step.type === "delay"
  ) {
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
          parseDraftObjectFields(step.dataFields, "Create record step data");
        } catch {
          addIssue({
            stepId: step.id,
            level: "error",
            message: "Create record step data contains an invalid field value.",
          });
        }
        continue;
      }

      if (step.type === "update_runtime_record") {
        if (step.entityLogicalName.trim().length === 0) {
          addIssue({
            stepId: step.id,
            level: "error",
            message: "Update record step is missing an entity logical name.",
          });
        }
        if (step.recordId.trim().length === 0) {
          addIssue({
            stepId: step.id,
            level: "error",
            message: "Update record step requires a record id.",
          });
        }
        try {
          parseDraftObjectFields(step.dataFields, "Update record step data");
        } catch {
          addIssue({
            stepId: step.id,
            level: "error",
            message: "Update record step data contains an invalid field value.",
          });
        }
        continue;
      }

      if (step.type === "delete_runtime_record") {
        if (step.entityLogicalName.trim().length === 0) {
          addIssue({
            stepId: step.id,
            level: "error",
            message: "Delete record step is missing an entity logical name.",
          });
        }
        if (step.recordId.trim().length === 0) {
          addIssue({
            stepId: step.id,
            level: "error",
            message: "Delete record step requires a record id.",
          });
        }
        continue;
      }

      if (step.type === "send_email") {
        if (step.to.trim().length === 0) {
          addIssue({
            stepId: step.id,
            level: "error",
            message: "Send email step requires a recipient address.",
          });
        }

        if (step.subject.trim().length === 0) {
          addIssue({
            stepId: step.id,
            level: "error",
            message: "Send email step requires a subject.",
          });
        }

        if (step.body.trim().length === 0) {
          addIssue({
            stepId: step.id,
            level: "error",
            message: "Send email step requires a message body.",
          });
        }

        continue;
      }

      if (step.type === "http_request") {
        if (step.method.trim().length === 0) {
          addIssue({
            stepId: step.id,
            level: "error",
            message: "HTTP request step requires a method.",
          });
        }

        if (step.url.trim().length === 0) {
          addIssue({
            stepId: step.id,
            level: "error",
            message: "HTTP request step requires a URL.",
          });
        }

        try {
          const parsed = JSON.parse(step.headersJson) as unknown;
          const isObject = parsed && typeof parsed === "object" && !Array.isArray(parsed);
          if (!isObject) {
            addIssue({
              stepId: step.id,
              level: "error",
              message: "HTTP request headers must be a JSON object.",
            });
          } else {
            for (const [key, value] of Object.entries(
              parsed as Record<string, unknown>,
            )) {
              if (typeof value !== "string") {
                addIssue({
                  stepId: step.id,
                  level: "error",
                  message: `HTTP request header "${key}" must be a string.`,
                });
              }
            }
          }
        } catch {
          addIssue({
            stepId: step.id,
            level: "error",
            message: "HTTP request headers contain invalid JSON.",
          });
        }

        try {
          const parsed = JSON.parse(step.headerSecretRefsJson) as unknown;
          const isObject = parsed && typeof parsed === "object" && !Array.isArray(parsed);
          if (!isObject) {
            addIssue({
              stepId: step.id,
              level: "error",
              message: "HTTP request secret headers must be a JSON object.",
            });
          } else {
            for (const [key, value] of Object.entries(parsed as Record<string, unknown>)) {
              if (typeof value !== "string") {
                addIssue({
                  stepId: step.id,
                  level: "error",
                  message: `HTTP request secret header "${key}" must be a string.`,
                });
              }
            }
          }
        } catch {
          addIssue({
            stepId: step.id,
            level: "error",
            message: "HTTP request secret headers contain invalid JSON.",
          });
        }

        if (step.bodyMode === "object") {
          try {
            parseDraftObjectFields(step.bodyFields, "HTTP request body");
          } catch (error) {
            addIssue({
              stepId: step.id,
              level: "error",
              message:
                error instanceof Error
                  ? error.message
                  : "HTTP request body contains an invalid field value.",
            });
          }
        } else if (step.bodyMode === "array") {
          try {
            parseDraftArrayItems(step.bodyArrayItems, "HTTP request body");
          } catch (error) {
            addIssue({
              stepId: step.id,
              level: "error",
              message:
                error instanceof Error
                  ? error.message
                  : "HTTP request body contains an invalid array item.",
            });
          }
        } else if (step.bodyMode === "scalar") {
          try {
            parseDraftValue(step.bodyScalarKind, step.bodyScalarValue, "HTTP request body");
          } catch (error) {
            addIssue({
              stepId: step.id,
              level: "error",
              message:
                error instanceof Error
                  ? error.message
                  : "HTTP request body contains an invalid scalar value.",
            });
          }
        } else if (step.bodyMode === "json") {
          try {
            JSON.parse(step.bodyJson);
          } catch {
            addIssue({
              stepId: step.id,
              level: "error",
              message: "HTTP request body must be valid JSON.",
            });
          }
        }

        continue;
      }

      if (step.type === "webhook") {
        if (step.endpoint.trim().length === 0) {
          addIssue({
            stepId: step.id,
            level: "error",
            message: "Webhook step requires an endpoint.",
          });
        }

        if (step.event.trim().length === 0) {
          addIssue({
            stepId: step.id,
            level: "error",
            message: "Webhook step requires an event name.",
          });
        }

        try {
          const parsed = JSON.parse(step.headersJson) as unknown;
          const isObject = parsed && typeof parsed === "object" && !Array.isArray(parsed);
          if (!isObject) {
            addIssue({
              stepId: step.id,
              level: "error",
              message: "Webhook headers must be a JSON object.",
            });
          } else {
            for (const [key, value] of Object.entries(
              parsed as Record<string, unknown>,
            )) {
              if (typeof value !== "string") {
                addIssue({
                  stepId: step.id,
                  level: "error",
                  message: `Webhook header "${key}" must be a string.`,
                });
              }
            }
          }
        } catch {
          addIssue({
            stepId: step.id,
            level: "error",
            message: "Webhook headers contain invalid JSON.",
          });
        }

        try {
          const parsed = JSON.parse(step.headerSecretRefsJson) as unknown;
          const isObject = parsed && typeof parsed === "object" && !Array.isArray(parsed);
          if (!isObject) {
            addIssue({
              stepId: step.id,
              level: "error",
              message: "Webhook secret headers must be a JSON object.",
            });
          } else {
            for (const [key, value] of Object.entries(parsed as Record<string, unknown>)) {
              if (typeof value !== "string") {
                addIssue({
                  stepId: step.id,
                  level: "error",
                  message: `Webhook secret header "${key}" must be a string.`,
                });
              }
            }
          }
        } catch {
          addIssue({
            stepId: step.id,
            level: "error",
            message: "Webhook secret headers contain invalid JSON.",
          });
        }

        try {
          parseDraftObjectFields(step.payloadFields, "Webhook payload");
        } catch {
          addIssue({
            stepId: step.id,
            level: "error",
            message: "Webhook payload contains an invalid field value.",
          });
        }

        continue;
      }

      if (step.type === "assign_owner") {
        if (step.entityLogicalName.trim().length === 0) {
          addIssue({
            stepId: step.id,
            level: "error",
            message: "Assign owner step is missing an entity logical name.",
          });
        }
        if (step.recordId.trim().length === 0) {
          addIssue({
            stepId: step.id,
            level: "error",
            message: "Assign owner step requires a record id.",
          });
        }
        if (step.ownerId.trim().length === 0) {
          addIssue({
            stepId: step.id,
            level: "error",
            message: "Assign owner step requires an owner id.",
          });
        }
        continue;
      }

      if (step.type === "approval_request") {
        if (step.entityLogicalName.trim().length === 0) {
          addIssue({
            stepId: step.id,
            level: "error",
            message: "Approval request step is missing an entity logical name.",
          });
        }
        if (step.recordId.trim().length === 0) {
          addIssue({
            stepId: step.id,
            level: "error",
            message: "Approval request step requires a record id.",
          });
        }
        if (step.requestType.trim().length === 0) {
          addIssue({
            stepId: step.id,
            level: "error",
            message: "Approval request step requires a request type.",
          });
        }
        try {
          parseDraftObjectFields(step.payloadFields, "Approval request payload");
        } catch {
          addIssue({
            stepId: step.id,
            level: "error",
            message: "Approval request payload contains an invalid field value.",
          });
        }
        continue;
      }

      if (step.type === "delay") {
        const parsed = Number.parseInt(step.durationMs, 10);
        if (!Number.isFinite(parsed) || parsed <= 0) {
          addIssue({
            stepId: step.id,
            level: "error",
            message: "Delay step requires a positive duration in milliseconds.",
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
          parseDraftValue(step.valueKind, step.valueText, "Condition value");
        } catch {
          addIssue({
            stepId: step.id,
            level: "error",
            message: "Condition value is invalid for the selected type.",
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
          : triggerType === "webhook_received"
            ? "Webhook trigger requires a webhook key."
            : triggerType === "form_submitted"
              ? "Form trigger requires a form key."
              : triggerType === "inbound_email_received"
                ? "Inbound email trigger requires a mailbox key."
                : triggerType === "approval_event_received"
                  ? "Approval trigger requires an approval key."
            : "Runtime record trigger requires an entity logical name.",
    });
  }

  validateBranch(steps);
  return issues;
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
      dataFields: createDraftObjectFieldsFromValue(step.data),
    };
  }

  if (step.type === "update_runtime_record") {
    return {
      id: createId(),
      type: "update_runtime_record",
      entityLogicalName: step.entity_logical_name,
      recordId: step.record_id,
      dataFields: createDraftObjectFieldsFromValue(step.data),
    };
  }

  if (step.type === "delete_runtime_record") {
    return {
      id: createId(),
      type: "delete_runtime_record",
      entityLogicalName: step.entity_logical_name,
      recordId: step.record_id,
    };
  }

  if (step.type === "send_email") {
    return {
      id: createId(),
      type: "send_email",
      to: step.to,
      subject: step.subject,
      body: step.body,
      htmlBody: step.html_body ?? "",
    };
  }

  if (step.type === "http_request") {
    const bodyMode =
      step.body == null
        ? "none"
        : isJsonObjectValue(step.body)
          ? "object"
          : isJsonArrayValue(step.body)
            ? "array"
            : "scalar";
    const bodyFields = isJsonObjectValue(step.body)
      ? createDraftObjectFieldsFromValue(step.body)
      : [];
    const bodyArrayItems = isJsonArrayValue(step.body)
      ? createDraftArrayItemsFromValue(step.body)
      : [];
    return {
      id: createId(),
      type: "http_request",
      method: step.method,
      url: step.url,
      headersJson: JSON.stringify(step.headers ?? {}, null, 2),
      headerSecretRefsJson: JSON.stringify(step.header_secret_refs ?? {}, null, 2),
      bodyMode,
      bodyFields,
      bodyArrayItems,
      bodyScalarKind:
        step.body != null && !isJsonObjectValue(step.body) && !isJsonArrayValue(step.body)
          ? inferDraftValueKind(step.body)
          : "string",
      bodyScalarValue:
        step.body != null && !isJsonObjectValue(step.body) && !isJsonArrayValue(step.body)
          ? stringifyDraftValue(step.body)
          : "",
      bodyJson: JSON.stringify(step.body ?? null, null, 2),
    };
  }

  if (step.type === "webhook") {
    return {
      id: createId(),
      type: "webhook",
      endpoint: step.endpoint,
      event: step.event,
      headersJson: JSON.stringify(step.headers ?? {}, null, 2),
      headerSecretRefsJson: JSON.stringify(step.header_secret_refs ?? {}, null, 2),
      payloadFields: createDraftObjectFieldsFromValue(step.payload),
    };
  }

  if (step.type === "assign_owner") {
    return {
      id: createId(),
      type: "assign_owner",
      entityLogicalName: step.entity_logical_name,
      recordId: step.record_id,
      ownerId: step.owner_id,
      reason: step.reason ?? "",
    };
  }

  if (step.type === "approval_request") {
    return {
      id: createId(),
      type: "approval_request",
      entityLogicalName: step.entity_logical_name,
      recordId: step.record_id,
      requestType: step.request_type,
      requestedBy: step.requested_by ?? "",
      approverId: step.approver_id ?? "",
      reason: step.reason ?? "",
      payloadFields: createDraftObjectFieldsFromValue(step.payload ?? {}),
    };
  }

  if (step.type === "delay") {
    return {
      id: createId(),
      type: "delay",
      durationMs: String(step.duration_ms),
      reason: step.reason ?? "",
    };
  }

  return {
    id: createId(),
    type: "condition",
    fieldPath: step.field_path,
    operator: step.operator,
    valueKind: step.operator === "exists" ? "null" : inferDraftValueKind(step.value ?? ""),
    valueText: step.operator === "exists" ? "" : stringifyDraftValue(step.value ?? ""),
    thenLabel: step.then_label ?? "Yes",
    elseLabel: step.else_label ?? "No",
    thenSteps: step.then_steps.map((nestedStep) => createDraftFromTransport(nestedStep, createId)),
    elseSteps: step.else_steps.map((nestedStep) => createDraftFromTransport(nestedStep, createId)),
  };
}

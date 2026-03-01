import { useEffect, useRef, useState } from "react";
import {
  Bell,
  ChevronDown,
  ChevronUp,
  CheckCircle2,
  Copy,
  Database,
  GitBranch,
  MessageSquare,
  Plus,
  Trash2,
  XCircle,
} from "lucide-react";

import { Button, Input, Label, Select, Textarea } from "@qryvanta/ui";

import { ExpressionBuilderPopover } from "@/components/automation/workflow-studio/panels/expression-builder-popover";
import { TriggerConfigPanel } from "@/components/automation/workflow-studio/panels/trigger-config-panel";
import {
  CONDITION_OPERATORS,
  summarizeStep,
  type CatalogInsertMode,
  type DraftConditionStep,
  type DraftWorkflowStep,
  type DynamicTokenOption,
  type TriggerType,
} from "@/components/automation/workflow-studio/model";
import type {
  RetryWorkflowStepStrategyDto,
  WorkflowConditionOperatorDto,
  WorkflowRunStepTraceResponse,
} from "@/lib/api";

type RetryPreset = "immediate" | "backoff_800" | "backoff_2000" | "backoff_5000";

function appendExpression(value: string, expression: string): string {
  return value.trim().length === 0 ? expression : `${value} ${expression}`;
}

function insertTokenMappingIntoJsonObject(jsonText: string, fieldPath: string): string {
  const trimmedFieldPath = fieldPath.trim();
  if (trimmedFieldPath.length === 0) {
    return jsonText;
  }

  const key =
    trimmedFieldPath.split(".").filter((segment) => segment.length > 0).at(-1) ??
    trimmedFieldPath;
  const token = `{{trigger.payload.${trimmedFieldPath}}}`;

  let base: Record<string, unknown> = {};
  try {
    const parsed = JSON.parse(jsonText) as unknown;
    if (parsed && typeof parsed === "object" && !Array.isArray(parsed)) {
      base = parsed as Record<string, unknown>;
    }
  } catch {
    base = {};
  }

  if (base[key] === undefined) {
    base[key] = token;
  }

  return JSON.stringify(base, null, 2);
}

function tokenChipsFromValue(value: string): string[] {
  const matches = value.match(/\{\{[^}]+\}\}/g);
  if (!matches) return [];
  return Array.from(new Set(matches));
}

type AutoMappedFieldPreview = {
  key: string;
  sourcePath: string;
};

function triggerPayloadMappedFieldsFromJson(dataJson: string): AutoMappedFieldPreview[] {
  const pattern = /"([^"]+)"\s*:\s*"\{\{\s*trigger\.payload\.([^}\s]+)\s*\}\}"/g;
  const matches = Array.from(dataJson.matchAll(pattern));
  if (matches.length === 0) {
    return [];
  }

  const previews: AutoMappedFieldPreview[] = [];
  const seen = new Set<string>();
  for (const match of matches) {
    const key = (match[1] ?? "").trim();
    const sourcePath = (match[2] ?? "").trim();
    if (key.length === 0 || sourcePath.length === 0) {
      continue;
    }

    const dedupeKey = `${key}::${sourcePath}`;
    if (seen.has(dedupeKey)) {
      continue;
    }
    seen.add(dedupeKey);
    previews.push({ key, sourcePath });
  }

  return previews;
}

function retryConfigFromPreset(preset: RetryPreset): {
  strategy: RetryWorkflowStepStrategyDto;
  backoffMs?: number;
} {
  if (preset === "immediate") return { strategy: "immediate" };
  if (preset === "backoff_2000") return { strategy: "backoff", backoffMs: 2000 };
  if (preset === "backoff_5000") return { strategy: "backoff", backoffMs: 5000 };
  return { strategy: "backoff", backoffMs: 800 };
}

function stepIcon(type: DraftWorkflowStep["type"]) {
  switch (type) {
    case "log_message":
      return <MessageSquare className="size-4" />;
    case "create_runtime_record":
      return <Database className="size-4" />;
    case "condition":
      return <GitBranch className="size-4" />;
  }
}

function stepIconBg(type: DraftWorkflowStep["type"]): string {
  switch (type) {
    case "log_message":
      return "bg-blue-100 text-blue-700";
    case "create_runtime_record":
      return "bg-sky-100 text-sky-700";
    case "condition":
      return "bg-amber-100 text-amber-700";
  }
}

function stepBorderColor(type: DraftWorkflowStep["type"]): string {
  switch (type) {
    case "log_message":
      return "border-blue-200";
    case "create_runtime_record":
      return "border-sky-200";
    case "condition":
      return "border-amber-200";
  }
}

function stepTypeLabel(type: DraftWorkflowStep["type"]): string {
  switch (type) {
    case "log_message":
      return "Log Message";
    case "create_runtime_record":
      return "Create Record";
    case "condition":
      return "Condition";
  }
}

function stepTypeLabelColor(type: DraftWorkflowStep["type"]): string {
  switch (type) {
    case "log_message":
      return "text-blue-700";
    case "create_runtime_record":
      return "text-sky-700";
    case "condition":
      return "text-amber-700";
  }
}

export function FlowConnector({ onAdd }: { onAdd: () => void }) {
  return (
    <div className="flex flex-col items-center">
      <div className="h-6 w-px bg-zinc-200" />
      <button
        type="button"
        className="flex size-6 items-center justify-center rounded-full border border-zinc-200 bg-white text-zinc-400 shadow-sm transition hover:border-emerald-300 hover:bg-emerald-50 hover:text-emerald-600 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-emerald-400"
        onClick={onAdd}
        title="Add step here"
      >
        <Plus className="size-3.5" />
      </button>
      <div className="h-6 w-px bg-zinc-200" />
    </div>
  );
}

function StepTraceStatus({ trace }: { trace: WorkflowRunStepTraceResponse | null }) {
  if (!trace) return null;

  const ok = trace.status === "succeeded";
  return (
    <div
      className={`mt-1 flex items-center gap-1 text-[10px] font-medium ${ok ? "text-emerald-600" : "text-red-600"}`}
    >
      {ok ? <CheckCircle2 className="size-3" /> : <XCircle className="size-3" />}
      <span className="capitalize">{trace.status}</span>
      {trace.duration_ms !== null && (
        <span className="text-zinc-400">· {String(trace.duration_ms)}ms</span>
      )}
    </div>
  );
}

type TriggerCardProps = {
  triggerType: TriggerType;
  triggerEntityLogicalName: string;
  isExpanded: boolean;
  onToggle: () => void;
  onTriggerTypeChange: (type: TriggerType) => void;
  onTriggerEntityChange: (entity: string) => void;
  runtimeEntityOptions: Array<{ value: string; label: string }>;
};

export function TriggerCard({
  triggerType,
  triggerEntityLogicalName,
  isExpanded,
  onToggle,
  onTriggerTypeChange,
  onTriggerEntityChange,
  runtimeEntityOptions,
}: TriggerCardProps) {
  const subtitle =
    triggerType === "manual"
      ? "Manual trigger"
      : triggerType === "schedule_tick"
        ? `Schedule tick · ${triggerEntityLogicalName.trim() || "schedule key not set"}`
        : `${
            triggerType === "runtime_record_updated"
              ? "Record updated"
              : triggerType === "runtime_record_deleted"
                ? "Record deleted"
                : "Record created"
          } · ${triggerEntityLogicalName.trim() || "entity not set"}`;

  return (
    <div
      className={`w-full overflow-hidden rounded-xl border bg-white shadow-sm transition-shadow ${
        isExpanded ? "border-emerald-300 shadow-md" : "border-emerald-200 hover:shadow"
      }`}
    >
      <button
        type="button"
        className="flex w-full items-center gap-3 p-4 text-left transition-colors hover:bg-emerald-50/40"
        onClick={onToggle}
      >
        <div className="flex size-9 shrink-0 items-center justify-center rounded-lg bg-emerald-100 text-emerald-700">
          <Bell className="size-4" />
        </div>
        <div className="min-w-0 flex-1">
          <p className="text-[10px] font-semibold uppercase tracking-[0.12em] text-emerald-700">
            Trigger
          </p>
          <p className="truncate text-sm font-medium text-zinc-900">{subtitle}</p>
        </div>
        <div className="shrink-0 text-zinc-400">
          {isExpanded ? <ChevronUp className="size-4" /> : <ChevronDown className="size-4" />}
        </div>
      </button>

      {isExpanded && (
        <div className="space-y-4 border-t border-emerald-100 bg-zinc-50/50 p-4">
          <TriggerConfigPanel
            triggerType={triggerType}
            triggerEntityLogicalName={triggerEntityLogicalName}
            runtimeEntityOptions={runtimeEntityOptions}
            onTriggerTypeChange={onTriggerTypeChange}
            onTriggerEntityChange={onTriggerEntityChange}
          />
        </div>
      )}
    </div>
  );
}

function TokenChips({ value }: { value: string }) {
  const chips = tokenChipsFromValue(value);
  if (chips.length === 0) return null;

  return (
    <div className="flex flex-wrap gap-1">
      {chips.map((chip) => (
        <span
          key={chip}
          className="rounded-full border border-emerald-200 bg-emerald-50 px-2 py-0.5 font-mono text-[10px] text-emerald-800"
        >
          {chip}
        </span>
      ))}
    </div>
  );
}

function FieldSuggestionPicker({
  title,
  helper,
  fields,
  onPick,
}: {
  title: string;
  helper: string;
  fields: string[];
  onPick: (fieldPath: string) => void;
}) {
  const [query, setQuery] = useState("");

  const filtered = query.trim().length
    ? fields.filter((fieldPath) =>
        fieldPath.toLowerCase().includes(query.trim().toLowerCase()),
      )
    : fields;

  if (fields.length === 0) {
    return null;
  }

  return (
    <details className="rounded-md border border-zinc-200 bg-white p-2">
      <summary className="cursor-pointer text-[11px] font-semibold text-zinc-700">{title}</summary>
      <div className="mt-2 space-y-2">
        <p className="text-[10px] text-zinc-500">{helper}</p>
        <Input
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          placeholder="Filter fields..."
          className="h-8 text-xs"
        />
        <div className="max-h-28 overflow-y-auto rounded border border-zinc-200 p-1">
          {filtered.length === 0 ? (
            <p className="px-2 py-1 text-[10px] text-zinc-400">No fields match.</p>
          ) : (
            <div className="flex flex-wrap gap-1">
              {filtered.slice(0, 40).map((fieldPath) => (
                <button
                  key={fieldPath}
                  type="button"
                  className="rounded border border-zinc-300 bg-zinc-50 px-2 py-1 font-mono text-[10px] text-zinc-700 hover:border-emerald-300 hover:bg-emerald-50 hover:text-emerald-700"
                  onClick={() => onPick(fieldPath)}
                >
                  {fieldPath}
                </button>
              ))}
            </div>
          )}
        </div>
      </div>
    </details>
  );
}

type UpdateFn = (updater: (step: DraftWorkflowStep) => DraftWorkflowStep) => void;

function LogMessageForm({
  step,
  availableTokens,
  onUpdate,
}: {
  step: Extract<DraftWorkflowStep, { type: "log_message" }>;
  availableTokens: DynamicTokenOption[];
  onUpdate: UpdateFn;
}) {
  return (
    <div className="space-y-3">
      <div className="space-y-1.5">
        <Label htmlFor={`msg_${step.id}`}>Message</Label>
        <Input
          id={`msg_${step.id}`}
          value={step.message}
          onChange={(e) =>
            onUpdate((s) =>
              s.type === "log_message" ? { ...s, message: e.target.value } : s,
            )
          }
          placeholder="Enter message or build an expression..."
        />
        {step.message.trim().length === 0 && (
          <p className="text-[11px] text-red-600">Message is required.</p>
        )}
        <TokenChips value={step.message} />
      </div>
      <ExpressionBuilderPopover
        title="Message Expression"
        currentValue={step.message}
        tokens={availableTokens}
        onInsertExpression={(expr) =>
          onUpdate((s) =>
            s.type === "log_message"
              ? { ...s, message: appendExpression(s.message, expr) }
              : s,
          )
        }
      />
    </div>
  );
}

function CreateRecordForm({
  step,
  availableTokens,
  runtimeEntityOptions,
  fieldPathSuggestions,
  focusedFieldKey,
  onFocusApplied,
  onUpdate,
}: {
  step: Extract<DraftWorkflowStep, { type: "create_runtime_record" }>;
  availableTokens: DynamicTokenOption[];
  runtimeEntityOptions: Array<{ value: string; label: string }>;
  fieldPathSuggestions: string[];
  focusedFieldKey: string | null;
  onFocusApplied: () => void;
  onUpdate: UpdateFn;
}) {
  const dataTextareaRef = useRef<HTMLTextAreaElement | null>(null);

  useEffect(() => {
    if (!focusedFieldKey) {
      return;
    }

    const textarea = dataTextareaRef.current;
    if (!textarea) {
      onFocusApplied();
      return;
    }

    const quotedKey = `"${focusedFieldKey}"`;
    const index = step.dataJson.indexOf(quotedKey);
    const fallbackIndex = index >= 0 ? index : step.dataJson.indexOf(focusedFieldKey);
    if (fallbackIndex < 0) {
      onFocusApplied();
      return;
    }

    textarea.focus();
    textarea.setSelectionRange(fallbackIndex, fallbackIndex + focusedFieldKey.length + 2);
    onFocusApplied();
  }, [focusedFieldKey, onFocusApplied, step.dataJson]);

  let dataError: string | null = null;
  try {
    const parsed = JSON.parse(step.dataJson) as unknown;
    if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
      dataError = "Must be a JSON object (e.g. { \"key\": \"value\" }).";
    }
  } catch {
    dataError = "Invalid JSON.";
  }

  return (
    <div className="space-y-3">
      <div className="space-y-1.5">
        <Label htmlFor={`entity_${step.id}`}>Entity logical name</Label>
        <Input
          id={`entity_${step.id}`}
          value={step.entityLogicalName}
          onChange={(e) =>
            onUpdate((s) =>
              s.type === "create_runtime_record"
                ? { ...s, entityLogicalName: e.target.value }
                : s,
            )
          }
          placeholder="contact, task, note..."
          list={`entity_suggestions_${step.id}`}
        />
        <datalist id={`entity_suggestions_${step.id}`}>
          {runtimeEntityOptions.map((entity) => (
            <option key={entity.value} value={entity.value} />
          ))}
        </datalist>
        {step.entityLogicalName.trim().length === 0 && (
          <p className="text-[11px] text-red-600">Entity name is required.</p>
        )}
        {fieldPathSuggestions.length > 0 ? (
          <p className="text-[11px] text-zinc-500">
            Known fields: {fieldPathSuggestions.slice(0, 8).join(", ")}
            {fieldPathSuggestions.length > 8 ? " ..." : ""}
          </p>
        ) : null}
        <FieldSuggestionPicker
          title="Field Mapping Picker"
          helper="Click a field to append a trigger token mapping into Data JSON."
          fields={fieldPathSuggestions}
          onPick={(fieldPath) =>
            onUpdate((currentStep) =>
              currentStep.type === "create_runtime_record"
                ? {
                    ...currentStep,
                    dataJson: insertTokenMappingIntoJsonObject(
                      currentStep.dataJson,
                      fieldPath,
                    ),
                  }
                : currentStep,
            )
          }
        />
      </div>
      <div className="space-y-1.5">
        <Label htmlFor={`data_${step.id}`}>Data (JSON object)</Label>
        <Textarea
          ref={dataTextareaRef}
          id={`data_${step.id}`}
          className="font-mono text-xs"
          rows={6}
          value={step.dataJson}
          onChange={(e) =>
            onUpdate((s) =>
              s.type === "create_runtime_record"
                ? { ...s, dataJson: e.target.value }
                : s,
            )
          }
        />
        {dataError && <p className="text-[11px] text-red-600">{dataError}</p>}
        <TokenChips value={step.dataJson} />
      </div>
      <ExpressionBuilderPopover
        title="Data Expression"
        currentValue={step.dataJson}
        tokens={availableTokens}
        onInsertExpression={(expr) =>
          onUpdate((s) =>
            s.type === "create_runtime_record"
              ? { ...s, dataJson: appendExpression(s.dataJson, expr) }
              : s,
          )
        }
      />
    </div>
  );
}

function ConditionForm({
  step,
  availableTokens,
  fieldPathSuggestions,
  onUpdate,
}: {
  step: Extract<DraftWorkflowStep, { type: "condition" }>;
  availableTokens: DynamicTokenOption[];
  fieldPathSuggestions: string[];
  onUpdate: UpdateFn;
}) {
  const [showLabels, setShowLabels] = useState(false);

  let valueError: string | null = null;
  if (step.operator !== "exists") {
    try {
      JSON.parse(step.valueJson);
    } catch {
      valueError = "Must be valid JSON (e.g. \"active\", 42, true).";
    }
  }

  return (
    <div className="space-y-3">
      <div className="grid grid-cols-3 items-end gap-2">
        <div className="space-y-1.5">
          <Label htmlFor={`field_${step.id}`}>Field path</Label>
          <Input
            id={`field_${step.id}`}
            value={step.fieldPath}
            onChange={(e) =>
              onUpdate((s) =>
                s.type === "condition" ? { ...s, fieldPath: e.target.value } : s,
              )
            }
            placeholder="payload.status"
            list={`field_path_suggestions_${step.id}`}
          />
          <datalist id={`field_path_suggestions_${step.id}`}>
            {fieldPathSuggestions.map((fieldPath) => (
              <option key={fieldPath} value={fieldPath} />
            ))}
          </datalist>
        </div>
        <div className="space-y-1.5">
          <Label htmlFor={`op_${step.id}`}>Operator</Label>
          <Select
            id={`op_${step.id}`}
            value={step.operator}
            onChange={(e) =>
              onUpdate((s) =>
                s.type === "condition"
                  ? { ...s, operator: e.target.value as WorkflowConditionOperatorDto }
                  : s,
              )
            }
          >
            {CONDITION_OPERATORS.map((op) => (
              <option key={op} value={op}>
                {op}
              </option>
            ))}
          </Select>
        </div>
        <div className="space-y-1.5">
          <Label htmlFor={`val_${step.id}`}>Value (JSON)</Label>
          <Input
            id={`val_${step.id}`}
            value={step.valueJson}
            disabled={step.operator === "exists"}
            onChange={(e) =>
              onUpdate((s) =>
                s.type === "condition" ? { ...s, valueJson: e.target.value } : s,
              )
            }
            placeholder='"active"'
          />
        </div>
      </div>
      {step.fieldPath.trim().length === 0 && (
        <p className="text-[11px] text-red-600">Field path is required.</p>
      )}
      {valueError && <p className="text-[11px] text-red-600">{valueError}</p>}
      <FieldSuggestionPicker
        title="Field Path Picker"
        helper="Pick a trigger field path to set this condition quickly."
        fields={fieldPathSuggestions}
        onPick={(fieldPath) =>
          onUpdate((currentStep) =>
            currentStep.type === "condition"
              ? { ...currentStep, fieldPath }
              : currentStep,
          )
        }
      />
      <ExpressionBuilderPopover
        title="Condition Value Expression"
        currentValue={step.valueJson}
        tokens={availableTokens}
        onInsertExpression={(expr) =>
          onUpdate((s) =>
            s.type === "condition"
              ? { ...s, valueJson: appendExpression(s.valueJson, expr) }
              : s,
          )
        }
      />
      <button
        type="button"
        className="text-[11px] text-zinc-500 underline-offset-2 hover:text-zinc-700 hover:underline"
        onClick={() => setShowLabels((v) => !v)}
      >
        {showLabels ? "Hide branch labels" : "Customize branch labels"}
      </button>
      {showLabels && (
        <div className="grid grid-cols-2 gap-2">
          <div className="space-y-1.5">
            <Label htmlFor={`then_${step.id}`}>Yes label</Label>
            <Input
              id={`then_${step.id}`}
              value={step.thenLabel}
              onChange={(e) =>
                onUpdate((s) =>
                  s.type === "condition" ? { ...s, thenLabel: e.target.value } : s,
                )
              }
              placeholder="Yes"
            />
          </div>
          <div className="space-y-1.5">
            <Label htmlFor={`else_${step.id}`}>No label</Label>
            <Input
              id={`else_${step.id}`}
              value={step.elseLabel}
              onChange={(e) =>
                onUpdate((s) =>
                  s.type === "condition" ? { ...s, elseLabel: e.target.value } : s,
                )
              }
              placeholder="No"
            />
          </div>
        </div>
      )}
    </div>
  );
}

function StepTraceDebug({
  trace,
  isRetryingStep,
  onRetryStep,
}: {
  trace: WorkflowRunStepTraceResponse;
  isRetryingStep: boolean;
  onRetryStep: (
    stepPath: string,
    strategy: RetryWorkflowStepStrategyDto,
    backoffMs?: number,
  ) => void;
}) {
  const [preset, setPreset] = useState<RetryPreset>("immediate");

  return (
    <div className="space-y-3 rounded-lg border border-zinc-200 bg-zinc-50 p-3">
      <div className="flex items-center justify-between">
        <p className="text-[10px] font-semibold uppercase tracking-[0.12em] text-zinc-500">
          Run debug
        </p>
        <span className="font-mono text-[10px] text-zinc-400">{trace.step_path}</span>
      </div>

      <div className="grid grid-cols-2 gap-2">
        <div className="space-y-1">
          <p className="text-[10px] font-semibold uppercase tracking-wide text-zinc-400">Input</p>
          <Textarea
            className="font-mono text-[10px]"
            rows={4}
            value={JSON.stringify(trace.input_payload, null, 2)}
            readOnly
          />
        </div>
        <div className="space-y-1">
          <p className="text-[10px] font-semibold uppercase tracking-wide text-zinc-400">Output</p>
          <Textarea
            className="font-mono text-[10px]"
            rows={4}
            value={JSON.stringify(trace.output_payload, null, 2)}
            readOnly
          />
        </div>
      </div>

      {trace.error_message && (
        <div className="space-y-2">
          <p className="rounded border border-red-200 bg-red-50 px-2 py-1.5 text-[11px] text-red-700">
            {trace.error_message}
          </p>
          <div className="flex items-center gap-2">
            <Select
              id={`retry_${trace.step_path}`}
              value={preset}
              onChange={(e) => setPreset(e.target.value as RetryPreset)}
            >
              <option value="immediate">Immediate retry</option>
              <option value="backoff_800">Backoff 0.8s</option>
              <option value="backoff_2000">Backoff 2s</option>
              <option value="backoff_5000">Backoff 5s</option>
            </Select>
            <Button
              type="button"
              size="sm"
              disabled={isRetryingStep}
              onClick={() => {
                const { strategy, backoffMs } = retryConfigFromPreset(preset);
                onRetryStep(trace.step_path, strategy, backoffMs);
              }}
            >
              {isRetryingStep ? "Retrying..." : "Retry step"}
            </Button>
          </div>
        </div>
      )}
    </div>
  );
}

type StepCardProps = {
  step: DraftWorkflowStep;
  isExpanded: boolean;
  trace: WorkflowRunStepTraceResponse | null;
  availableTokens: DynamicTokenOption[];
  runtimeEntityOptions: Array<{ value: string; label: string }>;
  triggerFieldPathSuggestions: string[];
  getEntityFieldPathSuggestions: (entityLogicalName: string) => string[];
  onToggle: () => void;
  onUpdate: UpdateFn;
  onRemove: () => void;
  onDuplicate: () => void;
  isRetryingStep: boolean;
  onRetryStep: (
    stepPath: string,
    strategy: RetryWorkflowStepStrategyDto,
    backoffMs?: number,
  ) => void;
};

function StepCard({
  step,
  isExpanded,
  trace,
  availableTokens,
  runtimeEntityOptions,
  triggerFieldPathSuggestions,
  getEntityFieldPathSuggestions,
  onToggle,
  onUpdate,
  onRemove,
  onDuplicate,
  isRetryingStep,
  onRetryStep,
}: StepCardProps) {
  const [focusedMappedFieldKey, setFocusedMappedFieldKey] = useState<string | null>(null);

  const autoMappedFields =
    step.type === "create_runtime_record"
      ? triggerPayloadMappedFieldsFromJson(step.dataJson)
      : [];

  return (
    <div
      className={`w-full overflow-hidden rounded-xl border bg-white shadow-sm transition-shadow ${stepBorderColor(step.type)} ${
        isExpanded ? "shadow-md" : "hover:shadow"
      }`}
    >
      <div className="flex items-center gap-3 px-4 py-3">
        <button
          type="button"
          className="flex min-w-0 flex-1 items-start gap-3 text-left"
          onClick={onToggle}
        >
          <div
            className={`mt-0.5 flex size-9 shrink-0 items-center justify-center rounded-lg ${stepIconBg(step.type)}`}
          >
            {stepIcon(step.type)}
          </div>
          <div className="min-w-0 flex-1">
            <p
              className={`text-[10px] font-semibold uppercase tracking-[0.12em] ${stepTypeLabelColor(step.type)}`}
            >
              {stepTypeLabel(step.type)}
            </p>
            <p className="truncate text-sm text-zinc-800">{summarizeStep(step)}</p>
            {autoMappedFields.length > 0 ? (
              <div className="mt-1 flex flex-wrap items-center gap-1">
                <span className="text-[10px] text-emerald-700">Mapped:</span>
                {autoMappedFields.slice(0, 5).map((mapped) => (
                  <button
                    key={`${mapped.key}:${mapped.sourcePath}`}
                    type="button"
                    className="rounded border border-emerald-300 bg-emerald-50 px-1.5 py-0.5 font-mono text-[10px] text-emerald-800 hover:bg-emerald-100"
                    title={`Map ${mapped.key} from trigger.payload.${mapped.sourcePath}`}
                    onClick={(event) => {
                      event.stopPropagation();
                      if (!isExpanded) {
                        onToggle();
                      }
                      setFocusedMappedFieldKey(mapped.key);
                    }}
                  >
                    {mapped.key}
                  </button>
                ))}
                {autoMappedFields.length > 5 ? (
                  <span className="text-[10px] text-zinc-500">...</span>
                ) : null}
              </div>
            ) : null}
            <StepTraceStatus trace={trace} />
          </div>
        </button>

        <div className="flex shrink-0 items-center gap-0.5">
          <button
            type="button"
            className="flex size-7 items-center justify-center rounded-md text-zinc-400 transition-colors hover:bg-zinc-100 hover:text-zinc-600"
            onClick={onDuplicate}
            title="Duplicate step"
          >
            <Copy className="size-3.5" />
          </button>
          <button
            type="button"
            className="flex size-7 items-center justify-center rounded-md text-zinc-400 transition-colors hover:bg-red-50 hover:text-red-600"
            onClick={onRemove}
            title="Remove step"
          >
            <Trash2 className="size-3.5" />
          </button>
          <button
            type="button"
            className={`flex size-7 items-center justify-center rounded-md transition-colors ${
              isExpanded
                ? "bg-zinc-100 text-zinc-600"
                : "text-zinc-400 hover:bg-zinc-100 hover:text-zinc-600"
            }`}
            onClick={onToggle}
            title={isExpanded ? "Collapse" : "Configure"}
          >
            {isExpanded ? <ChevronUp className="size-3.5" /> : <ChevronDown className="size-3.5" />}
          </button>
        </div>
      </div>

      {isExpanded && (
        <div className="space-y-4 border-t border-zinc-100 bg-zinc-50/40 p-4">
          {step.type === "log_message" && (
            <LogMessageForm step={step} availableTokens={availableTokens} onUpdate={onUpdate} />
          )}
          {step.type === "create_runtime_record" && (
            <CreateRecordForm
              step={step}
              availableTokens={availableTokens}
              runtimeEntityOptions={runtimeEntityOptions}
              fieldPathSuggestions={getEntityFieldPathSuggestions(step.entityLogicalName)}
              focusedFieldKey={focusedMappedFieldKey}
              onFocusApplied={() => setFocusedMappedFieldKey(null)}
              onUpdate={onUpdate}
            />
          )}
          {step.type === "condition" && (
            <ConditionForm
              step={step}
              availableTokens={availableTokens}
              fieldPathSuggestions={triggerFieldPathSuggestions}
              onUpdate={onUpdate}
            />
          )}
          {trace && (
            <StepTraceDebug
              trace={trace}
              isRetryingStep={isRetryingStep}
              onRetryStep={onRetryStep}
            />
          )}
        </div>
      )}
    </div>
  );
}

export type SharedStepProps = {
  expandedNodeId: string | null;
  onExpandNode: (id: string | null) => void;
  onUpdateStep: (
    stepId: string,
    updater: (s: DraftWorkflowStep) => DraftWorkflowStep,
  ) => void;
  onRemoveStep: (stepId: string) => void;
  onDuplicateStep: (stepId: string) => void;
  onOpenNodePicker: (mode: CatalogInsertMode, stepId?: string) => void;
  getAvailableTokens: (stepId: string) => DynamicTokenOption[];
  runtimeEntityOptions: Array<{ value: string; label: string }>;
  triggerFieldPathSuggestions: string[];
  getEntityFieldPathSuggestions: (entityLogicalName: string) => string[];
  stepTraceByPath: Record<string, WorkflowRunStepTraceResponse>;
  stepPathByStepId: Record<string, string>;
  isRetryingStep: boolean;
  onRetryStep: (
    stepPath: string,
    strategy: RetryWorkflowStepStrategyDto,
    backoffMs?: number,
  ) => void;
};

function BranchColumn({
  label,
  isYes,
  steps,
  conditionId,
  ...shared
}: SharedStepProps & {
  label: string;
  isYes: boolean;
  steps: DraftWorkflowStep[];
  conditionId: string;
}) {
  const addMode: CatalogInsertMode = isYes ? "then_selected" : "else_selected";

  return (
    <div className="flex min-w-0 flex-1 flex-col overflow-hidden rounded-xl border border-zinc-200 bg-zinc-50">
      <div
        className={`flex items-center gap-2 border-b border-zinc-200 px-3 py-2 ${
          isYes ? "bg-emerald-50" : "bg-red-50"
        }`}
      >
        <div className={`size-2 rounded-full ${isYes ? "bg-emerald-500" : "bg-red-400"}`} />
        <p
          className={`text-[11px] font-semibold uppercase tracking-[0.12em] ${
            isYes ? "text-emerald-700" : "text-red-700"
          }`}
        >
          {label || (isYes ? "Yes" : "No")}
        </p>
      </div>

      <div className="flex flex-col items-center gap-0 p-3">
        {steps.length === 0 ? (
          <p className="py-2 text-[11px] text-zinc-400">No steps yet</p>
        ) : (
          steps.map((step) => (
            <StepBlock key={step.id} step={step} {...shared} />
          ))
        )}
        <button
          type="button"
          className="mt-2 flex w-full items-center justify-center gap-1.5 rounded-lg border border-dashed border-zinc-300 px-3 py-2 text-[11px] text-zinc-400 transition hover:border-emerald-300 hover:bg-emerald-50 hover:text-emerald-600"
          onClick={() => shared.onOpenNodePicker(addMode, conditionId)}
        >
          <Plus className="size-3.5" />
          Add step
        </button>
      </div>
    </div>
  );
}

function ConditionBlock({
  step,
  ...shared
}: SharedStepProps & { step: DraftConditionStep }) {
  const isExpanded = shared.expandedNodeId === step.id;
  const stepPath = shared.stepPathByStepId[step.id];
  const trace = stepPath ? (shared.stepTraceByPath[stepPath] ?? null) : null;
  const tokens = shared.getAvailableTokens(step.id);

  return (
    <div className="w-full">
      <StepCard
        step={step}
        isExpanded={isExpanded}
        trace={trace}
        availableTokens={tokens}
        runtimeEntityOptions={shared.runtimeEntityOptions}
        triggerFieldPathSuggestions={shared.triggerFieldPathSuggestions}
        getEntityFieldPathSuggestions={shared.getEntityFieldPathSuggestions}
        onToggle={() => shared.onExpandNode(isExpanded ? null : step.id)}
        onUpdate={(updater) => shared.onUpdateStep(step.id, updater)}
        onRemove={() => shared.onRemoveStep(step.id)}
        onDuplicate={() => shared.onDuplicateStep(step.id)}
        isRetryingStep={shared.isRetryingStep}
        onRetryStep={shared.onRetryStep}
      />

      <div className="flex justify-center">
        <div className="h-4 w-px bg-zinc-200" />
      </div>

      <div className="flex gap-3">
        <BranchColumn
          label={step.thenLabel || "Yes"}
          isYes={true}
          steps={step.thenSteps}
          conditionId={step.id}
          {...shared}
        />
        <BranchColumn
          label={step.elseLabel || "No"}
          isYes={false}
          steps={step.elseSteps}
          conditionId={step.id}
          {...shared}
        />
      </div>

      <div className="flex justify-center">
        <div className="h-4 w-px bg-zinc-200" />
      </div>
    </div>
  );
}

export function StepBlock({ step, ...shared }: SharedStepProps & { step: DraftWorkflowStep }) {
  if (step.type === "condition") {
    return (
      <>
        <ConditionBlock step={step} {...shared} />
        <FlowConnector onAdd={() => shared.onOpenNodePicker("after_selected", step.id)} />
      </>
    );
  }

  const isExpanded = shared.expandedNodeId === step.id;
  const stepPath = shared.stepPathByStepId[step.id];
  const trace = stepPath ? (shared.stepTraceByPath[stepPath] ?? null) : null;
  const tokens = shared.getAvailableTokens(step.id);

  return (
    <>
      <StepCard
        step={step}
        isExpanded={isExpanded}
        trace={trace}
        availableTokens={tokens}
        runtimeEntityOptions={shared.runtimeEntityOptions}
        triggerFieldPathSuggestions={shared.triggerFieldPathSuggestions}
        getEntityFieldPathSuggestions={shared.getEntityFieldPathSuggestions}
        onToggle={() => shared.onExpandNode(isExpanded ? null : step.id)}
        onUpdate={(updater) => shared.onUpdateStep(step.id, updater)}
        onRemove={() => shared.onRemoveStep(step.id)}
        onDuplicate={() => shared.onDuplicateStep(step.id)}
        isRetryingStep={shared.isRetryingStep}
        onRetryStep={shared.onRetryStep}
      />
      <FlowConnector onAdd={() => shared.onOpenNodePicker("after_selected", step.id)} />
    </>
  );
}

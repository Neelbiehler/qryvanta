import type { FormEvent } from "react";
import {
  BadgeCheck,
  Bell,
  Braces,
  CalendarDays,
  CalendarSync,
  CheckCircle2,
  CircleDotDashed,
  CircleUserRound,
  Clock3,
  Database,
  ExternalLink,
  FileText,
  GitBranch,
  Globe,
  ListChecks,
  Mail,
  MessageSquareMore,
  Play,
  Search,
  TriangleAlert,
  type LucideIcon,
  UserRoundCheck,
  ShieldCheck,
  Siren,
} from "lucide-react";

import { Button, Input, Label, Select, Textarea } from "@qryvanta/ui";

import type { WorkflowResponse } from "@/lib/api";
import {
  STEP_LIBRARY,
  createDraftFieldId,
  parseDraftObjectFields,
  type DraftObjectField,
  type DraftValueKind,
  type DraftWorkflowStep,
  type FlowTemplateCategory,
  type FlowTemplateId,
  type TriggerType,
  type WorkflowValidationIssue,
} from "@/components/automation/workflow-studio/model";

export type TemplateOption = {
  id: FlowTemplateId;
  label: string;
  description: string;
  category: FlowTemplateCategory;
};

export type Tab = "actions" | "details" | "test";

type ActionsTabProps = {
  catalogQuery: string;
  onCatalogQueryChange: (value: string) => void;
  catalogCategory: "all" | FlowTemplateCategory;
  onCatalogCategoryChange: (value: "all" | FlowTemplateCategory) => void;
  filteredTemplates: TemplateOption[];
  onInsertTemplate: (templateId: FlowTemplateId) => void;
  onAddRootStep: (stepType: DraftWorkflowStep["type"]) => void;
};

type DetailsTabProps = {
  logicalName: string;
  onLogicalNameChange: (value: string) => void;
  displayName: string;
  onDisplayNameChange: (value: string) => void;
  description: string;
  onDescriptionChange: (value: string) => void;
  maxAttempts: string;
  onMaxAttemptsChange: (value: string) => void;
  workflowLifecycleState: WorkflowResponse["lifecycle_state"];
  publishedVersion: number | null;
  isSaving: boolean;
  isPublishing: boolean;
  isDisabling: boolean;
  validationIssues: WorkflowValidationIssue[];
  validationErrorCount: number;
  onFocusValidationIssue: (issue: WorkflowValidationIssue) => void;
  onSaveWorkflow: (event: FormEvent<HTMLFormElement>) => void;
  onPublishWorkflow: () => void;
  onDisableWorkflow: () => void;
};

type TestTabProps = {
  workflows: WorkflowResponse[];
  selectedWorkflow: string;
  selectedWorkflowDefinition: WorkflowResponse | null;
  selectedWorkflowTriggerSchema: {
    fields: Array<{ logical_name: string; display_name: string; field_type: string }>;
  } | null;
  executePayloadFields: DraftObjectField[];
  onExecutePayloadFieldsChange: (fields: DraftObjectField[]) => void;
  onLoadSuggestedExecutePayload: () => void;
  isExecuting: boolean;
  onExecuteWorkflow: (event: FormEvent<HTMLFormElement>) => void;
  onExecutionWorkflowChange: (workflowLogicalName: string) => void;
};

const TEMPLATE_ICONS: Partial<Record<FlowTemplateId, LucideIcon>> = {
  manual_trigger: Bell,
  record_created_trigger: CircleUserRound,
  webhook_trigger: Bell,
  inbound_email_trigger: Mail,
  form_submission_trigger: BadgeCheck,
  schedule_hourly_trigger: CalendarSync,
  schedule_daily_trigger: CalendarDays,
  approval_requested_trigger: ShieldCheck,
  condition_equals: GitBranch,
  condition_exists: GitBranch,
  http_request: Globe,
  dispatch_webhook: ExternalLink,
  send_email_notification: Mail,
  send_slack_notification: MessageSquareMore,
  transform_payload: Braces,
  delay_step: Clock3,
  create_task: ListChecks,
  create_followup_task: ListChecks,
  assign_record_owner: UserRoundCheck,
  create_approval_request: ShieldCheck,
  create_incident_ticket: Siren,
  upsert_contact_profile: CircleUserRound,
  create_note: FileText,
  post_feed_update: MessageSquareMore,
  create_audit_entry: ShieldCheck,
  log_info: Database,
  log_warning: Database,
};

const STEP_ICONS: Partial<Record<DraftWorkflowStep["type"], LucideIcon>> = {
  log_message: Database,
  create_runtime_record: CircleDotDashed,
  update_runtime_record: Database,
  delete_runtime_record: TriangleAlert,
  send_email: Mail,
  http_request: Globe,
  webhook: ExternalLink,
  assign_owner: UserRoundCheck,
  approval_request: ShieldCheck,
  delay: Clock3,
  condition: GitBranch,
};

const CATEGORY_CHIPS: Array<{ value: "all" | FlowTemplateCategory; label: string }> = [
  { value: "all", label: "All" },
  { value: "logic", label: "Logic" },
  { value: "data", label: "Data" },
  { value: "integration", label: "Integrations" },
  { value: "operations", label: "Operations" },
  { value: "trigger", label: "Triggers" },
];

function payloadValuePlaceholder(valueKind: DraftValueKind): string {
  switch (valueKind) {
    case "string":
      return "value";
    case "number":
      return "42";
    case "boolean":
      return "true";
    case "null":
      return "null";
    case "json":
      return '{\n  "nested": true\n}';
  }
}

function TriggerPayloadEditor({
  fields,
  onChange,
}: {
  fields: DraftObjectField[];
  onChange: (fields: DraftObjectField[]) => void;
}) {
  function addField() {
    onChange([
      ...fields,
      {
        id: createDraftFieldId(),
        key: "",
        valueKind: "string",
        value: "",
      },
    ]);
  }

  function updateField(
    fieldId: string,
    patch: Partial<Pick<DraftObjectField, "key" | "valueKind" | "value">>,
  ) {
    onChange(
      fields.map((field) =>
        field.id === fieldId
          ? {
              ...field,
              ...patch,
              value:
                patch.valueKind && patch.valueKind !== field.valueKind
                  ? patch.valueKind === "boolean"
                    ? "true"
                    : patch.valueKind === "null"
                      ? ""
                      : field.value
                  : patch.value ?? field.value,
            }
          : field,
      ),
    );
  }

  function removeField(fieldId: string) {
    onChange(fields.filter((field) => field.id !== fieldId));
  }

  return (
    <div className="space-y-2 rounded-lg border border-zinc-200 bg-zinc-50/60 p-3">
      <div className="flex items-center justify-between">
        <Label htmlFor="wb_test_payload_key_0" className="text-xs">
          Sample payload fields
        </Label>
        <Button type="button" size="sm" variant="outline" onClick={addField}>
          Add field
        </Button>
      </div>
      {fields.length === 0 ? (
        <p className="text-[11px] text-zinc-500">No payload fields yet.</p>
      ) : (
        fields.map((field, index) => (
          <div key={field.id} className="space-y-2 rounded-md border border-zinc-200 bg-white p-2.5">
            <div className="grid grid-cols-[1.2fr_0.8fr_auto] gap-2">
              <Input
                id={`wb_test_payload_key_${index}`}
                value={field.key}
                onChange={(event) => updateField(field.id, { key: event.target.value })}
                placeholder="field_name"
              />
              <Select
                id={`wb_test_payload_kind_${index}`}
                value={field.valueKind}
                onChange={(event) =>
                  updateField(field.id, {
                    valueKind: event.target.value as DraftValueKind,
                  })
                }
              >
                <option value="string">Text</option>
                <option value="number">Number</option>
                <option value="boolean">Boolean</option>
                <option value="null">Null</option>
                <option value="json">JSON</option>
              </Select>
              <Button type="button" size="sm" variant="ghost" onClick={() => removeField(field.id)}>
                Remove
              </Button>
            </div>
            {field.valueKind === "boolean" ? (
              <Select
                id={`wb_test_payload_value_${index}`}
                value={field.value === "false" ? "false" : "true"}
                onChange={(event) => updateField(field.id, { value: event.target.value })}
              >
                <option value="true">True</option>
                <option value="false">False</option>
              </Select>
            ) : field.valueKind === "null" ? (
              <p className="rounded border border-dashed border-zinc-200 px-2.5 py-2 text-[11px] text-zinc-500">
                This field will be sent as `null`.
              </p>
            ) : field.valueKind === "json" ? (
              <Textarea
                id={`wb_test_payload_value_${index}`}
                className="font-mono text-xs"
                rows={4}
                value={field.value}
                onChange={(event) => updateField(field.id, { value: event.target.value })}
                placeholder={payloadValuePlaceholder(field.valueKind)}
              />
            ) : (
              <Input
                id={`wb_test_payload_value_${index}`}
                value={field.value}
                onChange={(event) => updateField(field.id, { value: event.target.value })}
                placeholder={payloadValuePlaceholder(field.valueKind)}
              />
            )}
          </div>
        ))
      )}
    </div>
  );
}

export function TabButton({
  active,
  onClick,
  badge,
  children,
}: {
  active: boolean;
  onClick: () => void;
  badge?: number;
  children: React.ReactNode;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={`relative flex flex-1 items-center justify-center gap-1.5 py-2.5 text-xs font-medium transition ${
        active
          ? "border-b-2 border-emerald-600 text-emerald-700"
          : "border-b-2 border-transparent text-zinc-500 hover:text-zinc-800"
      }`}
    >
      {children}
      {badge !== undefined && (
        <span className="absolute right-1 top-1 flex size-4 items-center justify-center rounded-full bg-red-500 text-[9px] font-bold text-white">
          {badge > 9 ? "9+" : badge}
        </span>
      )}
    </button>
  );
}

export function ActionsTab({
  catalogQuery,
  onCatalogQueryChange,
  catalogCategory,
  onCatalogCategoryChange,
  filteredTemplates,
  onInsertTemplate,
  onAddRootStep,
}: ActionsTabProps) {
  return (
    <div className="flex flex-col gap-3 p-3">
      <div className="relative">
        <Search className="absolute left-2.5 top-1/2 size-3.5 -translate-y-1/2 text-zinc-400" />
        <input
          className="w-full rounded-lg border border-zinc-200 bg-zinc-50 py-2 pl-8 pr-3 text-sm placeholder-zinc-400 outline-none transition focus:border-emerald-400 focus:bg-white focus:ring-2 focus:ring-emerald-100"
          placeholder="Search actions..."
          value={catalogQuery}
          onChange={(e) => onCatalogQueryChange(e.target.value)}
          autoComplete="off"
        />
      </div>

      <div className="flex flex-wrap gap-1">
        {CATEGORY_CHIPS.map((chip) => (
          <button
            key={chip.value}
            type="button"
            onClick={() => onCatalogCategoryChange(chip.value)}
            className={`rounded-full px-2.5 py-1 text-[11px] font-medium transition ${
              catalogCategory === chip.value
                ? "bg-emerald-100 text-emerald-700 ring-1 ring-emerald-300"
                : "bg-zinc-100 text-zinc-600 hover:bg-zinc-200"
            }`}
          >
            {chip.label}
          </button>
        ))}
      </div>

      {filteredTemplates.length > 0 ? (
        <div className="space-y-1">
          {filteredTemplates.map((template) => {
            const Icon = TEMPLATE_ICONS[template.id] ?? Database;
            return (
              <button
                key={template.id}
                type="button"
                className="flex w-full items-start gap-3 rounded-lg border border-transparent px-2.5 py-2 text-left transition hover:border-emerald-200 hover:bg-emerald-50"
                onClick={() => onInsertTemplate(template.id)}
              >
                <span className="mt-0.5 flex size-7 shrink-0 items-center justify-center rounded-md bg-zinc-100 text-zinc-600">
                  <Icon className="size-3.5" />
                </span>
                <span className="min-w-0">
                  <span className="block text-xs font-semibold text-zinc-800">{template.label}</span>
                  <span className="block text-[11px] leading-tight text-zinc-500">{template.description}</span>
                </span>
              </button>
            );
          })}
        </div>
      ) : (
        <p className="py-6 text-center text-xs text-zinc-400">No matching actions</p>
      )}

      <div className="border-t border-zinc-100 pt-2">
        <p className="mb-2 text-[10px] font-semibold uppercase tracking-wider text-zinc-400">
          Quick add
        </p>
        <div className="flex flex-wrap gap-1.5">
          {STEP_LIBRARY.map((entry) => {
            const Icon = STEP_ICONS[entry.type] ?? CircleDotDashed;
            return (
              <button
                key={entry.type}
                type="button"
                className="flex items-center gap-1.5 rounded-md border border-zinc-200 bg-white px-2.5 py-1.5 text-xs font-medium text-zinc-700 transition hover:border-emerald-300 hover:bg-emerald-50 hover:text-emerald-700"
                onClick={() => onAddRootStep(entry.type)}
              >
                <Icon className="size-3" />
                {entry.label}
              </button>
            );
          })}
        </div>
      </div>
    </div>
  );
}

export function DetailsTab({
  logicalName,
  onLogicalNameChange,
  displayName,
  onDisplayNameChange,
  description,
  onDescriptionChange,
  maxAttempts,
  onMaxAttemptsChange,
  workflowLifecycleState,
  publishedVersion,
  isSaving,
  isPublishing,
  isDisabling,
  validationIssues,
  validationErrorCount,
  onFocusValidationIssue,
  onSaveWorkflow,
  onPublishWorkflow,
  onDisableWorkflow,
}: DetailsTabProps) {
  const lifecycleTone =
    workflowLifecycleState === "published"
      ? "bg-emerald-100 text-emerald-700"
      : workflowLifecycleState === "disabled"
        ? "bg-zinc-200 text-zinc-700"
        : "bg-amber-100 text-amber-700";
  const lifecycleLabel =
    workflowLifecycleState === "published"
      ? "Published"
      : workflowLifecycleState === "disabled"
        ? "Disabled"
        : "Draft";

  return (
    <form className="space-y-4 p-3" onSubmit={onSaveWorkflow}>
      <div className="space-y-3">
        <div className="space-y-1">
          <Label htmlFor="wb_display_name" className="text-xs">
            Name
          </Label>
          <Input
            id="wb_display_name"
            value={displayName}
            onChange={(e) => onDisplayNameChange(e.target.value)}
            placeholder="When a deal is created"
            required
          />
        </div>
        <div className="space-y-1">
          <Label htmlFor="wb_description" className="text-xs">
            Description
          </Label>
          <Input
            id="wb_description"
            value={description}
            onChange={(e) => onDescriptionChange(e.target.value)}
            placeholder="What this automation does"
          />
        </div>
        <div className="space-y-1">
          <Label htmlFor="wb_logical_name" className="text-xs">
            Logical name
          </Label>
          <Input
            id="wb_logical_name"
            value={logicalName}
            onChange={(e) => onLogicalNameChange(e.target.value)}
            placeholder="my_workflow"
            className="font-mono text-xs"
            required
          />
        </div>
        <div className="flex items-center gap-3">
          <div className="flex-1 space-y-1">
            <Label htmlFor="wb_max_attempts" className="text-xs">
              Retry attempts
            </Label>
            <Input
              id="wb_max_attempts"
              type="number"
              min={1}
              max={10}
              value={maxAttempts}
              onChange={(e) => onMaxAttemptsChange(e.target.value)}
              required
            />
          </div>
          <div className="space-y-1 self-end pb-0.5 text-right">
            <p className="text-[10px] font-semibold uppercase tracking-wider text-zinc-400">
              Release
            </p>
            <div className="flex items-center justify-end gap-2">
              <span className={`rounded-full px-2 py-0.5 text-[10px] font-semibold ${lifecycleTone}`}>
                {lifecycleLabel}
              </span>
              <span className="text-[11px] text-zinc-500">
                {publishedVersion === null ? "No published version" : `v${publishedVersion}`}
              </span>
            </div>
          </div>
        </div>
      </div>

      <div className="space-y-1.5 border-t border-zinc-100 pt-3">
        <p className="text-[10px] font-semibold uppercase tracking-wider text-zinc-400">
          Flow Checker
        </p>
        {validationIssues.length === 0 ? (
          <div className="flex items-center gap-2 text-xs text-emerald-700">
            <CheckCircle2 className="size-3.5" />
            No issues
          </div>
        ) : (
          <div className="space-y-1">
            {validationIssues.map((issue) => (
              <button
                key={issue.id}
                type="button"
                onClick={() => onFocusValidationIssue(issue)}
                className={`flex w-full items-start gap-2 rounded-md px-2 py-1.5 text-left transition ${
                  issue.level === "error"
                    ? "bg-red-50 text-red-700 hover:bg-red-100"
                    : "bg-amber-50 text-amber-700 hover:bg-amber-100"
                }`}
              >
                <TriangleAlert className="mt-0.5 size-3 shrink-0" />
                <span className="text-[11px] leading-tight">{issue.message}</span>
              </button>
            ))}
          </div>
        )}
      </div>

      <div className="space-y-2 border-t border-zinc-100 pt-3">
        <Button
          type="submit"
          className="w-full"
          disabled={isSaving || validationErrorCount > 0}
        >
          {isSaving ? "Saving..." : validationErrorCount > 0 ? "Fix errors to save" : "Save draft"}
        </Button>
        <Button
          type="button"
          className="w-full"
          variant="outline"
          disabled={isPublishing || validationErrorCount > 0}
          onClick={onPublishWorkflow}
        >
          {isPublishing ? "Publishing..." : validationErrorCount > 0 ? "Fix errors to publish" : "Publish draft"}
        </Button>
        <Button
          type="button"
          className="w-full"
          variant="ghost"
          disabled={isDisabling || publishedVersion === null}
          onClick={onDisableWorkflow}
        >
          {isDisabling ? "Disabling..." : "Disable published workflow"}
        </Button>
      </div>
    </form>
  );
}

export function TestTab({
  workflows,
  selectedWorkflow,
  selectedWorkflowDefinition,
  selectedWorkflowTriggerSchema,
  executePayloadFields,
  onExecutePayloadFieldsChange,
  onLoadSuggestedExecutePayload,
  isExecuting,
  onExecuteWorkflow,
  onExecutionWorkflowChange,
}: TestTabProps) {
  let payloadPreview = "{}";
  let payloadPreviewError: string | null = null;
  try {
    payloadPreview = JSON.stringify(
      parseDraftObjectFields(executePayloadFields, "Trigger payload"),
      null,
      2,
    );
  } catch (error) {
    payloadPreviewError =
      error instanceof Error ? error.message : "Trigger payload contains invalid field values.";
  }

  const triggerType = (selectedWorkflowDefinition?.trigger_type ?? "manual") as TriggerType;
  const triggerLabel =
    triggerType === "runtime_record_created"
      ? "Record created"
      : triggerType === "runtime_record_updated"
        ? "Record updated"
        : triggerType === "runtime_record_deleted"
          ? "Record deleted"
          : triggerType === "schedule_tick"
            ? "Schedule tick"
            : triggerType === "webhook_received"
              ? "Webhook received"
              : triggerType === "form_submitted"
                ? "Form submitted"
                : triggerType === "inbound_email_received"
                  ? "Inbound email"
                  : triggerType === "approval_event_received"
                    ? "Approval event"
                    : "Manual";

  return (
    <form className="space-y-4 p-3" onSubmit={onExecuteWorkflow}>
      <div className="space-y-1">
        <p className="text-xs text-zinc-500">
          Run this flow with a sample trigger payload to verify behavior before deploying.
        </p>
      </div>
      <div className="space-y-1">
        <Label htmlFor="wb_test_workflow" className="text-xs">
          Workflow
        </Label>
        <Select
          id="wb_test_workflow"
          value={selectedWorkflow}
          onChange={(e) => onExecutionWorkflowChange(e.target.value)}
        >
          <option value="">Select workflow</option>
          {workflows.map((wf) => (
            <option key={wf.logical_name} value={wf.logical_name}>
              {wf.display_name}
            </option>
          ))}
        </Select>
      </div>
      <div className="space-y-2 rounded-lg border border-zinc-200 bg-zinc-50 p-3">
        <div className="flex items-center justify-between gap-2">
          <div>
            <p className="text-[10px] font-semibold uppercase tracking-wider text-zinc-400">
              Trigger Shape
            </p>
            <p className="text-xs text-zinc-600">
              {triggerLabel}
              {selectedWorkflowDefinition?.trigger_entity_logical_name
                ? ` · ${selectedWorkflowDefinition.trigger_entity_logical_name}`
                : ""}
            </p>
          </div>
          <Button type="button" size="sm" variant="outline" onClick={onLoadSuggestedExecutePayload}>
            Load sample
          </Button>
        </div>
        {selectedWorkflowTriggerSchema ? (
          <p className="text-[11px] text-zinc-500">
            Published fields:{" "}
            {selectedWorkflowTriggerSchema.fields
              .slice(0, 8)
              .map((field) => field.display_name || field.logical_name)
              .join(", ")}
            {selectedWorkflowTriggerSchema.fields.length > 8 ? " ..." : ""}
          </p>
        ) : null}
      </div>
      <TriggerPayloadEditor
        fields={executePayloadFields}
        onChange={onExecutePayloadFieldsChange}
      />
      <div className="space-y-1">
        <Label htmlFor="wb_test_payload_preview" className="text-xs">
          Payload preview
        </Label>
        <Textarea
          id="wb_test_payload_preview"
          className="font-mono text-xs"
          value={payloadPreview}
          readOnly
          rows={8}
        />
        {payloadPreviewError ? (
          <p className="text-[11px] text-red-600">{payloadPreviewError}</p>
        ) : null}
      </div>
      <Button
        type="submit"
        variant="outline"
        className="w-full"
        disabled={isExecuting || !selectedWorkflow || payloadPreviewError !== null}
      >
        {isExecuting ? (
          <span className="flex items-center gap-2">
            <Play className="size-3.5 animate-pulse" />
            Running...
          </span>
        ) : (
          <span className="flex items-center gap-2">
            <Play className="size-3.5" />
            Run test
          </span>
        )}
      </Button>
    </form>
  );
}

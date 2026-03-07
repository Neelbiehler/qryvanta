import type {
  DraftWorkflowStep,
  TriggerType,
} from "@/components/automation/workflow-studio/model";
import { createDraftObjectFieldsFromValue } from "@/components/automation/workflow-studio/model";

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
    type: "update_runtime_record",
    label: "Update record",
    description: "Update an existing runtime record.",
  },
  {
    type: "delete_runtime_record",
    label: "Delete record",
    description: "Delete an existing runtime record.",
  },
  {
    type: "send_email",
    label: "Send email",
    description: "Send an outbound email notification.",
  },
  {
    type: "http_request",
    label: "HTTP request",
    description: "Call an external HTTP endpoint.",
  },
  {
    type: "webhook",
    label: "Webhook",
    description: "Dispatch an outbound webhook event.",
  },
  {
    type: "assign_owner",
    label: "Assign owner",
    description: "Route a record to an owner or queue.",
  },
  {
    type: "approval_request",
    label: "Approval request",
    description: "Create a native approval request.",
  },
  {
    type: "delay",
    label: "Delay",
    description: "Pause workflow execution for a bounded duration.",
  },
  {
    type: "condition",
    label: "Condition",
    description: "Branch into Yes/No step paths.",
  },
];

export type FlowTemplateId =
  | "manual_trigger"
  | "record_created_trigger"
  | "webhook_trigger"
  | "inbound_email_trigger"
  | "form_submission_trigger"
  | "schedule_hourly_trigger"
  | "schedule_daily_trigger"
  | "approval_requested_trigger"
  | "log_info"
  | "log_warning"
  | "post_feed_update"
  | "create_followup_task"
  | "assign_record_owner"
  | "create_approval_request"
  | "send_email_notification"
  | "send_slack_notification"
  | "dispatch_webhook"
  | "create_incident_ticket"
  | "upsert_contact_profile"
  | "create_audit_entry"
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
    id: "record_created_trigger",
    label: "Record Created Trigger",
    description: "Starts when a runtime record is created in the selected entity.",
    category: "trigger",
    keywords: ["trigger", "record", "created", "event", "start"],
    target: "trigger",
  },
  {
    id: "webhook_trigger",
    label: "Webhook Event Trigger",
    description: "Starts when the native workflow webhook ingress receives a matching key.",
    category: "trigger",
    keywords: ["trigger", "webhook", "event", "start"],
    target: "trigger",
  },
  {
    id: "inbound_email_trigger",
    label: "Inbound Email Trigger",
    description: "Starts when the native workflow email ingress receives a matching mailbox key.",
    category: "trigger",
    keywords: ["trigger", "email", "inbound", "mailbox", "start"],
    target: "trigger",
  },
  {
    id: "form_submission_trigger",
    label: "Form Submission Trigger",
    description: "Starts when the native workflow form ingress receives a matching key.",
    category: "trigger",
    keywords: ["trigger", "form", "submission", "event", "start"],
    target: "trigger",
  },
  {
    id: "schedule_hourly_trigger",
    label: "Hourly Schedule Trigger",
    description: "Starts when a schedule_hourly runtime tick record is created.",
    category: "trigger",
    keywords: ["trigger", "schedule", "hourly", "timer", "cron"],
    target: "trigger",
  },
  {
    id: "schedule_daily_trigger",
    label: "Daily Schedule Trigger",
    description: "Starts when a schedule_daily runtime tick record is created.",
    category: "trigger",
    keywords: ["trigger", "schedule", "daily", "timer", "cron"],
    target: "trigger",
  },
  {
    id: "approval_requested_trigger",
    label: "Approval Requested Trigger",
    description: "Starts when the native workflow approval ingress receives a matching approval key.",
    category: "trigger",
    keywords: ["trigger", "approval", "review", "request", "start"],
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
    description: "Call an external HTTP endpoint with a typed request step.",
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
    description: "Pause workflow execution with a native delay step.",
    category: "integration",
    keywords: ["delay", "wait", "timer"],
    target: "step",
  },
  {
    id: "send_email_notification",
    label: "Send Email Notification",
    description: "Send an outbound email with native workflow delivery.",
    category: "integration",
    keywords: ["email", "notification", "message", "integration"],
    target: "step",
  },
  {
    id: "send_slack_notification",
    label: "Send Slack Notification",
    description: "Create a chat_notification record for Slack/Teams relays.",
    category: "integration",
    keywords: ["slack", "teams", "chat", "notification", "integration"],
    target: "step",
  },
  {
    id: "dispatch_webhook",
    label: "Dispatch Webhook",
    description: "Send an outbound webhook event with native workflow delivery.",
    category: "integration",
    keywords: ["webhook", "dispatch", "http", "integration"],
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
    id: "create_followup_task",
    label: "Create Follow-up Task",
    description: "Create a task assigned for next-step follow-up work.",
    category: "data",
    keywords: ["task", "follow-up", "assign", "work"],
    target: "step",
  },
  {
    id: "assign_record_owner",
    label: "Assign Record Owner",
    description: "Assign a record to an owner or queue with a native action.",
    category: "data",
    keywords: ["assign", "owner", "routing", "queue"],
    target: "step",
  },
  {
    id: "create_approval_request",
    label: "Create Approval Request",
    description: "Create a native approval request for a target record.",
    category: "data",
    keywords: ["approval", "review", "request", "workflow"],
    target: "step",
  },
  {
    id: "create_incident_ticket",
    label: "Create Incident Ticket",
    description: "Create an incident_ticket record for operations handling.",
    category: "data",
    keywords: ["incident", "ticket", "ops", "support"],
    target: "step",
  },
  {
    id: "upsert_contact_profile",
    label: "Upsert Contact Profile",
    description: "Create a contact_upsert_queue record for profile syncing.",
    category: "data",
    keywords: ["contact", "crm", "profile", "sync", "upsert"],
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
  {
    id: "post_feed_update",
    label: "Post Feed Update",
    description: "Create a team_feed_event record for activity timelines.",
    category: "operations",
    keywords: ["feed", "activity", "post", "timeline"],
    target: "step",
  },
  {
    id: "create_audit_entry",
    label: "Create Audit Entry",
    description: "Create a workflow_audit_log record for compliance tracing.",
    category: "operations",
    keywords: ["audit", "compliance", "trace", "log"],
    target: "step",
  },
];

const SEARCH_SYNONYMS: Record<string, string[]> = {
  condition: ["if", "branch", "rule", "decision"],
  if: ["condition", "branch", "rule", "decision"],
  branch: ["condition", "if", "rule"],
  decision: ["condition", "if", "branch"],
  webhook: ["http", "event", "trigger"],
  email: ["mail", "notification", "message", "inbound"],
  slack: ["teams", "chat", "message", "notification"],
  schedule: ["timer", "cron", "daily", "hourly"],
  approval: ["review", "signoff", "request"],
  incident: ["ticket", "alert", "ops"],
  owner: ["assign", "routing", "queue"],
  webhook_dispatch: ["webhook", "http", "integration"],
  trigger: ["start", "when", "event"],
  action: ["step", "task", "operation"],
  task: ["todo", "work item", "follow-up"],
  note: ["comment", "activity", "log"],
  delay: ["wait", "pause", "sleep"],
  wait: ["delay", "timer", "pause"],
  transform: ["map", "shape", "convert"],
  map: ["transform", "convert", "shape"],
  http: ["api", "request", "webhook"],
  record: ["row", "entity", "data"],
  create: ["add", "insert", "new"],
  exists: ["present", "has", "available"],
  equals: ["is", "match", "same"],
};

function tokenizeSearchQuery(value: string): string[] {
  return value
    .split(/\s+/)
    .map((token) => token.trim())
    .filter((token) => token.length > 0);
}

function expandSearchTokens(tokens: string[]): string[] {
  const expanded = new Set<string>(tokens);
  for (const token of tokens) {
    const synonyms = SEARCH_SYNONYMS[token];
    if (!synonyms) {
      continue;
    }

    for (const synonym of synonyms) {
      expanded.add(synonym);
    }
  }

  return Array.from(expanded);
}

type TriggerTemplateConfig = {
  triggerType: TriggerType;
  triggerEntityLogicalName: string;
  statusLabel: string;
};

export function triggerTemplateConfig(templateId: FlowTemplateId): TriggerTemplateConfig | null {
  switch (templateId) {
    case "manual_trigger":
      return {
        triggerType: "manual",
        triggerEntityLogicalName: "",
        statusLabel: "Manual",
      };
    case "record_created_trigger":
      return {
        triggerType: "runtime_record_created",
        triggerEntityLogicalName: "contact",
        statusLabel: "Record Created",
      };
    case "webhook_trigger":
      return {
        triggerType: "webhook_received",
        triggerEntityLogicalName: "incoming_webhook",
        statusLabel: "Webhook Event",
      };
    case "inbound_email_trigger":
      return {
        triggerType: "inbound_email_received",
        triggerEntityLogicalName: "support",
        statusLabel: "Inbound Email",
      };
    case "form_submission_trigger":
      return {
        triggerType: "form_submitted",
        triggerEntityLogicalName: "lead_capture",
        statusLabel: "Form Submission",
      };
    case "schedule_hourly_trigger":
      return {
        triggerType: "schedule_tick",
        triggerEntityLogicalName: "hourly",
        statusLabel: "Hourly Schedule",
      };
    case "schedule_daily_trigger":
      return {
        triggerType: "schedule_tick",
        triggerEntityLogicalName: "daily",
        statusLabel: "Daily Schedule",
      };
    case "approval_requested_trigger":
      return {
        triggerType: "approval_event_received",
        triggerEntityLogicalName: "manager_signoff",
        statusLabel: "Approval Event",
      };
    default:
      return null;
  }
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
      dataFields: createDraftObjectFieldsFromValue({ title: "Follow-up" }),
    };
  }

  if (stepType === "update_runtime_record") {
    return {
      id: createId(),
      type: "update_runtime_record",
      entityLogicalName: "task",
      recordId: "{{trigger.payload.record_id}}",
      dataFields: createDraftObjectFieldsFromValue({ status: "in_progress" }),
    };
  }

  if (stepType === "delete_runtime_record") {
    return {
      id: createId(),
      type: "delete_runtime_record",
      entityLogicalName: "task",
      recordId: "{{trigger.payload.record_id}}",
    };
  }

  if (stepType === "send_email") {
    return {
      id: createId(),
      type: "send_email",
      to: "ops@example.com",
      subject: "Workflow alert",
      body: "Workflow {{run.id}} completed.",
      htmlBody: "",
    };
  }

  if (stepType === "http_request") {
    const defaultBody = {
      run_id: "{{run.id}}",
      record_id: "{{trigger.payload.record_id}}",
    };
    return {
      id: createId(),
      type: "http_request",
      method: "POST",
      url: "https://api.example.com/hooks/workflow",
      headersJson: JSON.stringify({ "content-type": "application/json" }, null, 2),
      headerSecretRefsJson: JSON.stringify({}, null, 2),
      bodyMode: "object",
      bodyFields: createDraftObjectFieldsFromValue(defaultBody),
      bodyArrayItems: [],
      bodyScalarKind: "string",
      bodyScalarValue: "",
      bodyJson: JSON.stringify(defaultBody, null, 2),
    };
  }

  if (stepType === "webhook") {
    return {
      id: createId(),
      type: "webhook",
      endpoint: "https://example.org/workflow-callback",
      event: "workflow.completed",
      headersJson: JSON.stringify({}, null, 2),
      headerSecretRefsJson: JSON.stringify({}, null, 2),
      payloadFields: createDraftObjectFieldsFromValue({
        run_id: "{{run.id}}",
        trigger_record_id: "{{trigger.payload.record_id}}",
      }),
    };
  }

  if (stepType === "assign_owner") {
    return {
      id: createId(),
      type: "assign_owner",
      entityLogicalName: "task",
      recordId: "{{trigger.payload.record_id}}",
      ownerId: "triage_queue",
      reason: "workflow routing",
    };
  }

  if (stepType === "approval_request") {
    return {
      id: createId(),
      type: "approval_request",
      entityLogicalName: "task",
      recordId: "{{trigger.payload.record_id}}",
      requestType: "record_change",
      requestedBy: "{{trigger.payload.triggered_by}}",
      approverId: "",
      reason: "Please review this record change.",
      payloadFields: createDraftObjectFieldsFromValue({ status: "pending_review" }),
    };
  }

  if (stepType === "delay") {
    return {
      id: createId(),
      type: "delay",
      durationMs: "5000",
      reason: "wait for downstream consistency",
    };
  }

  return {
    id: createId(),
    type: "condition",
    fieldPath: "status",
    operator: "equals",
    valueKind: "string",
    valueText: "open",
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
    case "record_created_trigger":
    case "webhook_trigger":
    case "inbound_email_trigger":
    case "form_submission_trigger":
    case "schedule_hourly_trigger":
    case "schedule_daily_trigger":
    case "approval_requested_trigger":
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
    case "post_feed_update":
      return {
        id: createId(),
        type: "create_runtime_record",
        entityLogicalName: "team_feed_event",
        dataFields: createDraftObjectFieldsFromValue({
          title: "Workflow update",
          body: "Processed {{trigger.payload.record_id}} in run {{run.id}}",
          visibility: "team",
        }),
      };
    case "create_audit_entry":
      return {
        id: createId(),
        type: "create_runtime_record",
        entityLogicalName: "workflow_audit_log",
        dataFields: createDraftObjectFieldsFromValue({
          run_id: "{{run.id}}",
          event: "workflow_step_completed",
          source_record_id: "{{trigger.payload.record_id}}",
        }),
      };
    case "create_task":
      return {
        id: createId(),
        type: "create_runtime_record",
        entityLogicalName: "task",
        dataFields: createDraftObjectFieldsFromValue({
          title: "Follow-up",
          priority: "normal",
        }),
      };
    case "create_note":
      return {
        id: createId(),
        type: "create_runtime_record",
        entityLogicalName: "note",
        dataFields: createDraftObjectFieldsFromValue({
          title: "Activity Note",
          body: "auto generated",
        }),
      };
    case "create_followup_task":
      return {
        id: createId(),
        type: "create_runtime_record",
        entityLogicalName: "task",
        dataFields: createDraftObjectFieldsFromValue({
          title: "Follow up on {{trigger.payload.record_id}}",
          status: "open",
          priority: "normal",
          source: "workflow",
        }),
      };
    case "assign_record_owner":
      return {
        id: createId(),
        type: "assign_owner",
        entityLogicalName: "{{trigger.payload.entity_logical_name}}",
        recordId: "{{trigger.payload.record_id}}",
        ownerId: "triage_queue",
        reason: "auto routing",
      };
    case "create_approval_request":
      return {
        id: createId(),
        type: "approval_request",
        entityLogicalName: "{{trigger.payload.entity_logical_name}}",
        recordId: "{{trigger.payload.record_id}}",
        requestType: "record_change",
        requestedBy: "{{trigger.payload.triggered_by}}",
        approverId: "",
        reason: "Review workflow-triggered change",
        payloadFields: createDraftObjectFieldsFromValue({
          trigger_record_id: "{{trigger.payload.record_id}}",
        }),
      };
    case "create_incident_ticket":
      return {
        id: createId(),
        type: "create_runtime_record",
        entityLogicalName: "incident_ticket",
        dataFields: createDraftObjectFieldsFromValue({
          title: "Automation incident for {{trigger.payload.record_id}}",
          severity: "medium",
          source: "workflow",
          status: "open",
        }),
      };
    case "upsert_contact_profile":
      return {
        id: createId(),
        type: "create_runtime_record",
        entityLogicalName: "contact_upsert_queue",
        dataFields: createDraftObjectFieldsFromValue({
          external_id: "{{trigger.payload.record_id}}",
          source: "workflow",
          payload: {
            email: "{{trigger.payload.email}}",
            name: "{{trigger.payload.name}}",
          },
        }),
      };
    case "http_request":
      const defaultBody = {
        run_id: "{{run.id}}",
        record_id: "{{trigger.payload.record_id}}",
      };
      return {
        id: createId(),
        type: "http_request",
        method: "POST",
        url: "https://api.example.com/hooks/workflow",
        headersJson: JSON.stringify(
          {
            "content-type": "application/json",
          },
          null,
          2,
        ),
        headerSecretRefsJson: JSON.stringify({}, null, 2),
        bodyMode: "object",
        bodyFields: createDraftObjectFieldsFromValue(defaultBody),
        bodyArrayItems: [],
        bodyScalarKind: "string",
        bodyScalarValue: "",
        bodyJson: JSON.stringify(defaultBody, null, 2),
      };
    case "transform_payload":
      return {
        id: createId(),
        type: "create_runtime_record",
        entityLogicalName: "integration_payload",
        dataFields: createDraftObjectFieldsFromValue({
          source: "trigger",
          transformed: true,
          mapping_version: "v1",
        }),
      };
    case "delay_step":
      return {
        id: createId(),
        type: "delay",
        durationMs: "300000",
        reason: "downstream consistency wait",
      };
    case "send_email_notification":
      return {
        id: createId(),
        type: "send_email",
        to: "ops@example.com",
        subject: "Workflow alert: {{trigger.payload.record_id}}",
        body: "Flow {{run.id}} processed {{trigger.payload.record_id}}.",
        htmlBody: "",
      };
    case "send_slack_notification":
      return {
        id: createId(),
        type: "create_runtime_record",
        entityLogicalName: "chat_notification",
        dataFields: createDraftObjectFieldsFromValue({
          provider: "slack",
          channel: "#ops-alerts",
          message: "Workflow {{run.id}} handled {{trigger.payload.record_id}}",
        }),
      };
    case "dispatch_webhook":
      return {
        id: createId(),
        type: "webhook",
        endpoint: "https://example.org/workflow-callback",
        event: "workflow.completed",
        headersJson: JSON.stringify({}, null, 2),
        headerSecretRefsJson: JSON.stringify({}, null, 2),
        payloadFields: createDraftObjectFieldsFromValue({
          run_id: "{{run.id}}",
          trigger_record_id: "{{trigger.payload.record_id}}",
        }),
      };
    case "condition_exists":
      return {
        id: createId(),
        type: "condition",
        fieldPath: "contact.email",
        operator: "exists",
        valueKind: "null",
        valueText: "",
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
            dataFields: createDraftObjectFieldsFromValue({ title: "Collect missing email" }),
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
        valueKind: "string",
        valueText: "open",
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
  const queryTokens = tokenizeSearchQuery(query);
  const expandedQueryTokens = expandSearchTokens(queryTokens);

  const withScore = FLOW_TEMPLATES.map((template) => {
    if (category !== "all" && template.category !== category) {
      return null;
    }

    const haystacks = [template.label, template.description, ...template.keywords].map((value) =>
      value.toLowerCase(),
    );

    if (!query) {
      return { template, score: 0 };
    }

    let score = 0;
    for (const token of expandedQueryTokens) {
      for (const haystack of haystacks) {
        if (haystack === token) {
          score += 120;
        } else if (haystack.startsWith(token)) {
          score += 90;
        } else if (haystack.includes(token)) {
          score += 45;
        } else {
          const charsMatched = token.split("").every((char) => haystack.includes(char));
          if (charsMatched) {
            score += 15;
          }
        }
      }
    }

    if (queryTokens.length > 1) {
      const allTokensPresent = queryTokens.every((token) =>
        haystacks.some((haystack) => haystack.includes(token)),
      );
      if (allTokensPresent) {
        score += 60;
      }
    }

    if (score === 0) {
      return null;
    }

    return { template, score };
  })
    .filter(
      (entry): entry is { template: (typeof FLOW_TEMPLATES)[number]; score: number } =>
        Boolean(entry),
    )
    .sort((left, right) => right.score - left.score || left.template.label.localeCompare(right.template.label));

  return withScore.map((entry) => entry.template);
}

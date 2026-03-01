import type {
  DraftWorkflowStep,
  TriggerType,
} from "@/components/automation/workflow-studio/model";

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
    description: "Starts when a webhook_event runtime record is created.",
    category: "trigger",
    keywords: ["trigger", "webhook", "event", "start"],
    target: "trigger",
  },
  {
    id: "inbound_email_trigger",
    label: "Inbound Email Trigger",
    description: "Starts when an inbound_email runtime record is captured.",
    category: "trigger",
    keywords: ["trigger", "email", "inbound", "mailbox", "start"],
    target: "trigger",
  },
  {
    id: "form_submission_trigger",
    label: "Form Submission Trigger",
    description: "Starts when a form_submission runtime record is created.",
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
    description: "Starts when an approval_request runtime record is created.",
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
    description: "Queue an outbound HTTP dispatch record for an integration worker.",
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
    id: "send_email_notification",
    label: "Send Email Notification",
    description: "Create an email_outbox record for downstream mail delivery.",
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
    description: "Create a webhook_dispatch record for outbound webhook delivery.",
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
    description: "Create a record_assignment event for ownership routing.",
    category: "data",
    keywords: ["assign", "owner", "routing", "queue"],
    target: "step",
  },
  {
    id: "create_approval_request",
    label: "Create Approval Request",
    description: "Create an approval_request record for human approval flow.",
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
        triggerType: "runtime_record_created",
        triggerEntityLogicalName: "webhook_event",
        statusLabel: "Webhook Event",
      };
    case "inbound_email_trigger":
      return {
        triggerType: "runtime_record_created",
        triggerEntityLogicalName: "inbound_email",
        statusLabel: "Inbound Email",
      };
    case "form_submission_trigger":
      return {
        triggerType: "runtime_record_created",
        triggerEntityLogicalName: "form_submission",
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
        triggerType: "runtime_record_created",
        triggerEntityLogicalName: "approval_request",
        statusLabel: "Approval Requested",
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
        dataJson: JSON.stringify(
          {
            title: "Workflow update",
            body: "Processed {{trigger.payload.record_id}} in run {{run.id}}",
            visibility: "team",
          },
          null,
          2,
        ),
      };
    case "create_audit_entry":
      return {
        id: createId(),
        type: "create_runtime_record",
        entityLogicalName: "workflow_audit_log",
        dataJson: JSON.stringify(
          {
            run_id: "{{run.id}}",
            event: "workflow_step_completed",
            source_record_id: "{{trigger.payload.record_id}}",
          },
          null,
          2,
        ),
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
    case "create_followup_task":
      return {
        id: createId(),
        type: "create_runtime_record",
        entityLogicalName: "task",
        dataJson: JSON.stringify(
          {
            title: "Follow up on {{trigger.payload.record_id}}",
            status: "open",
            priority: "normal",
            source: "workflow",
          },
          null,
          2,
        ),
      };
    case "assign_record_owner":
      return {
        id: createId(),
        type: "create_runtime_record",
        entityLogicalName: "record_assignment",
        dataJson: JSON.stringify(
          {
            source_record_id: "{{trigger.payload.record_id}}",
            source_entity: "{{trigger.payload.entity_logical_name}}",
            owner_id: "triage_queue",
            reason: "auto routing",
          },
          null,
          2,
        ),
      };
    case "create_approval_request":
      return {
        id: createId(),
        type: "create_runtime_record",
        entityLogicalName: "approval_request",
        dataJson: JSON.stringify(
          {
            request_type: "record_change",
            source_record_id: "{{trigger.payload.record_id}}",
            requested_by: "{{trigger.payload.triggered_by}}",
            status: "pending",
          },
          null,
          2,
        ),
      };
    case "create_incident_ticket":
      return {
        id: createId(),
        type: "create_runtime_record",
        entityLogicalName: "incident_ticket",
        dataJson: JSON.stringify(
          {
            title: "Automation incident for {{trigger.payload.record_id}}",
            severity: "medium",
            source: "workflow",
            status: "open",
          },
          null,
          2,
        ),
      };
    case "upsert_contact_profile":
      return {
        id: createId(),
        type: "create_runtime_record",
        entityLogicalName: "contact_upsert_queue",
        dataJson: JSON.stringify(
          {
            external_id: "{{trigger.payload.record_id}}",
            source: "workflow",
            payload: {
              email: "{{trigger.payload.email}}",
              name: "{{trigger.payload.name}}",
            },
          },
          null,
          2,
        ),
      };
    case "http_request":
      return {
        id: createId(),
        type: "create_runtime_record",
        entityLogicalName: "integration_http_request",
        dataJson: JSON.stringify(
          {
            method: "POST",
            url: "https://api.example.com/hooks/workflow",
            headers: {
              "content-type": "application/json",
            },
            body: {
              run_id: "{{run.id}}",
              record_id: "{{trigger.payload.record_id}}",
            },
          },
          null,
          2,
        ),
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
        type: "create_runtime_record",
        entityLogicalName: "workflow_delay_request",
        dataJson: JSON.stringify(
          {
            duration: "PT5M",
            reason: "downstream consistency wait",
            run_id: "{{run.id}}",
          },
          null,
          2,
        ),
      };
    case "send_email_notification":
      return {
        id: createId(),
        type: "create_runtime_record",
        entityLogicalName: "email_outbox",
        dataJson: JSON.stringify(
          {
            to: "ops@example.com",
            subject: "Workflow alert: {{trigger.payload.record_id}}",
            body: "Flow {{run.id}} processed {{trigger.payload.record_id}}.",
            channel: "email",
          },
          null,
          2,
        ),
      };
    case "send_slack_notification":
      return {
        id: createId(),
        type: "create_runtime_record",
        entityLogicalName: "chat_notification",
        dataJson: JSON.stringify(
          {
            provider: "slack",
            channel: "#ops-alerts",
            message: "Workflow {{run.id}} handled {{trigger.payload.record_id}}",
          },
          null,
          2,
        ),
      };
    case "dispatch_webhook":
      return {
        id: createId(),
        type: "create_runtime_record",
        entityLogicalName: "webhook_dispatch",
        dataJson: JSON.stringify(
          {
            endpoint: "https://example.org/workflow-callback",
            event: "workflow.completed",
            payload: {
              run_id: "{{run.id}}",
              trigger_record_id: "{{trigger.payload.record_id}}",
            },
          },
          null,
          2,
        ),
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

import type { LucideIcon } from "lucide-react";
import { Blocks, Briefcase, Shield, Workflow } from "lucide-react";

type ShowcaseNavGroup = {
  label: string;
  items: string[];
};

type ShowcaseFrame = {
  id: string;
  frame: string;
  badge: string;
  surface: string;
  route: string;
  capture: string;
  title: string;
  summary: string;
  narration: string;
  lane: string;
  quickActions: string[];
  navGroups: ShowcaseNavGroup[];
  activeNav: string;
  breadcrumbs: string[];
  metrics: Array<{
    label: string;
    value: string;
  }>;
  rows: Array<{
    primary: string;
    context: string;
    status: string;
    tone: "success" | "warning" | "critical" | "neutral";
  }>;
  events: string[];
  accent: string;
  glow: string;
  icon: LucideIcon;
  tone: "success" | "warning" | "critical" | "neutral";
};

export const showcaseFrames: ShowcaseFrame[] = [
  {
    id: "admin-governance",
    frame: "Scene 01",
    badge: "Admin",
    surface: "Admin Center",
    route: "/admin/account",
    capture: "00:00 - 00:06",
    title: "Security Settings",
    summary:
      "Based on the admin surface in apps/web. Shows role assignment and audit trail access.",
    narration:
      "Start in Admin Center to lock tenant posture before scaling any runtime workflows.",
    lane: "Tenant Guardrails",
    quickActions: ["Create Role", "Export Audit", "Enforce MFA"],
    breadcrumbs: ["Admin Center", "Security", "Settings"],
    navGroups: [
      { label: "Governance", items: ["Overview", "Roles", "Audit Log"] },
      { label: "Security", items: ["Security Settings", "MFA Policy"] },
    ],
    activeNav: "Security Settings",
    metrics: [
      { label: "Roles", value: "14" },
      { label: "Audit Events", value: "12k" },
      { label: "MFA", value: "92%" },
    ],
    rows: [
      {
        primary: "Tenant Administrator",
        context: "Role assignment",
        status: "Active",
        tone: "success",
      },
      {
        primary: "Audit export / finance",
        context: "Compliance trail",
        status: "Complete",
        tone: "neutral",
      },
      {
        primary: "Suspicious sign-in policy",
        context: "Security rule",
        status: "Watching",
        tone: "warning",
      },
    ],
    events: [
      "New role override request queued",
      "Audit export completed for finance tenant",
    ],
    accent: "#059669",
    glow: "rgba(5, 150, 105, 0.2)",
    icon: Shield,
    tone: "success",
  },
  {
    id: "maker-modeling",
    frame: "Scene 02",
    badge: "Maker",
    surface: "Maker Center",
    route: "/maker/entities",
    capture: "00:06 - 00:12",
    title: "Entity Library",
    summary:
      "Based on the Maker surface in apps/web. Entity schemas and their publish lifecycle.",
    narration:
      "Switch to Maker Center to define metadata once and publish contracts for every surface.",
    lane: "Metadata Studio",
    quickActions: ["Add Field", "Validate Draft", "Publish Schema"],
    breadcrumbs: ["Maker Center", "Entities"],
    navGroups: [
      {
        label: "Metadata",
        items: ["Overview", "Entities", "Views", "Forms"],
      },
      { label: "App Studio", items: ["Sitemap", "Publish"] },
    ],
    activeNav: "Entities",
    metrics: [
      { label: "Entities", value: "27" },
      { label: "Fields", value: "314" },
      { label: "Drafts", value: "4" },
    ],
    rows: [
      {
        primary: "contract",
        context: "13 fields · v9",
        status: "Published",
        tone: "success",
      },
      {
        primary: "invoice",
        context: "11 fields · v5",
        status: "Draft",
        tone: "warning",
      },
      {
        primary: "subscription",
        context: "8 fields · v4",
        status: "Published",
        tone: "neutral",
      },
    ],
    events: [
      "Contract schema draft updated",
      "Publish check passed with zero violations",
    ],
    accent: "#2563eb",
    glow: "rgba(37, 99, 235, 0.2)",
    icon: Blocks,
    tone: "warning",
  },
  {
    id: "worker-operations",
    frame: "Scene 03",
    badge: "Worker",
    surface: "Revenue App",
    route: "/worker/apps/revenue/contracts",
    capture: "00:12 - 00:18",
    title: "Contracts",
    summary:
      "Based on the Worker surface in apps/web. App navigation and record-level operations.",
    narration:
      "Move into Worker Apps where teams execute daily tasks on the same trusted model.",
    lane: "Operational Runtime",
    quickActions: ["New Record", "Bulk Edit", "Export View"],
    breadcrumbs: ["Worker", "Revenue App", "Contracts"],
    navGroups: [
      {
        label: "Sales",
        items: ["Contracts", "Invoices", "Opportunities"],
      },
      { label: "Customers", items: ["Accounts", "Contacts"] },
    ],
    activeNav: "Contracts",
    metrics: [
      { label: "Open Records", value: "146" },
      { label: "SLA Risk", value: "7" },
      { label: "Views", value: "19" },
    ],
    rows: [
      {
        primary: "Revenue App",
        context: "Assigned app",
        status: "Open",
        tone: "success",
      },
      {
        primary: "Contract renewal #8391",
        context: "Due in 1d",
        status: "At Risk",
        tone: "critical",
      },
      {
        primary: "New customer onboarding",
        context: "Task queue",
        status: "In Progress",
        tone: "warning",
      },
    ],
    events: [
      "Renewal task resolved by owner",
      "Escalation alert sent to manager queue",
    ],
    accent: "#0f766e",
    glow: "rgba(15, 118, 110, 0.2)",
    icon: Briefcase,
    tone: "neutral",
  },
  {
    id: "automation-loop",
    frame: "Scene 04",
    badge: "Automation",
    surface: "Maker Center",
    route: "/maker/automation/workflows",
    capture: "00:18 - 00:24",
    title: "Workflows",
    summary:
      "Use the same automation concepts as the web canvas: triggers, steps, branches, and run history.",
    narration:
      "Finish with automation loops that handle retries and event-driven operations.",
    lane: "Workflow Orchestration",
    quickActions: ["Run Manually", "Open Inspector", "Publish Flow"],
    breadcrumbs: ["Maker Center", "Automation", "Workflows"],
    navGroups: [
      {
        label: "Metadata",
        items: ["Overview", "Entities", "Views", "Forms"],
      },
      { label: "Automation", items: ["Workflows", "Schedules", "Run History"] },
    ],
    activeNav: "Workflows",
    metrics: [
      { label: "Active Flows", value: "22" },
      { label: "Runs Today", value: "3,214" },
      { label: "Retries", value: "18" },
    ],
    rows: [
      {
        primary: "invoice_overdue_followup",
        context: "Trigger: runtime_record_created",
        status: "Running",
        tone: "success",
      },
      {
        primary: "security_alert_triage",
        context: "Condition branch",
        status: "Retry 2/3",
        tone: "warning",
      },
      {
        primary: "daily_sync_digest",
        context: "Scheduled run",
        status: "Queued",
        tone: "neutral",
      },
    ],
    events: [
      "Workflow retry succeeded on attempt two",
      "New invoice reminder flow published",
    ],
    accent: "#c2410c",
    glow: "rgba(194, 65, 12, 0.18)",
    icon: Workflow,
    tone: "critical",
  },
];

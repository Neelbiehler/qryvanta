"use client";

import { useEffect, useRef, useState, type RefObject } from "react";
import {
  BadgeCheck,
  Bell,
  Braces,
  CalendarDays,
  CalendarSync,
  CircleUserRound,
  ChevronRight,
  Clock3,
  Database,
  ExternalLink,
  FileText,
  GitBranch,
  Globe,
  ListChecks,
  Mail,
  MessageSquareMore,
  ShieldCheck,
  Search,
  Siren,
  UserRoundCheck,
  X,
  type LucideIcon,
} from "lucide-react";

import type {
  CatalogInsertMode,
  FlowTemplateCategory,
  FlowTemplateId,
} from "@/components/automation/workflow-studio/model";

type TemplateOption = {
  id: FlowTemplateId;
  label: string;
  description: string;
  category: FlowTemplateCategory;
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

const TEMPLATE_COLORS: Partial<Record<FlowTemplateId, string>> = {
  condition_equals: "bg-amber-100 text-amber-700",
  condition_exists: "bg-amber-100 text-amber-700",
  log_info: "bg-blue-100 text-blue-700",
  log_warning: "bg-orange-100 text-orange-700",
  create_task: "bg-emerald-100 text-emerald-700",
  create_note: "bg-emerald-100 text-emerald-700",
  http_request: "bg-violet-100 text-violet-700",
  dispatch_webhook: "bg-violet-100 text-violet-700",
  send_email_notification: "bg-violet-100 text-violet-700",
  send_slack_notification: "bg-violet-100 text-violet-700",
  manual_trigger: "bg-emerald-100 text-emerald-700",
  record_created_trigger: "bg-emerald-100 text-emerald-700",
  webhook_trigger: "bg-emerald-100 text-emerald-700",
  inbound_email_trigger: "bg-emerald-100 text-emerald-700",
  form_submission_trigger: "bg-emerald-100 text-emerald-700",
  schedule_hourly_trigger: "bg-emerald-100 text-emerald-700",
  schedule_daily_trigger: "bg-emerald-100 text-emerald-700",
  approval_requested_trigger: "bg-emerald-100 text-emerald-700",
  create_followup_task: "bg-emerald-100 text-emerald-700",
  assign_record_owner: "bg-emerald-100 text-emerald-700",
  create_approval_request: "bg-emerald-100 text-emerald-700",
  create_incident_ticket: "bg-emerald-100 text-emerald-700",
  upsert_contact_profile: "bg-emerald-100 text-emerald-700",
  post_feed_update: "bg-blue-100 text-blue-700",
  create_audit_entry: "bg-blue-100 text-blue-700",
};

const CATEGORY_CHIPS: Array<{ value: "all" | FlowTemplateCategory; label: string }> = [
  { value: "all", label: "All" },
  { value: "logic", label: "Logic" },
  { value: "data", label: "Data" },
  { value: "integration", label: "Integrations" },
  { value: "operations", label: "Operations" },
  { value: "trigger", label: "Triggers" },
];

const INSERT_MODE_LABELS: Record<CatalogInsertMode, string> = {
  root: "main path",
  after_selected: "after selected step",
  then_selected: "into Yes branch",
  else_selected: "into No branch",
};

type NodePickerDialogProps = {
  open: boolean;
  inputRef: RefObject<HTMLInputElement | null>;
  query: string;
  category: "all" | FlowTemplateCategory;
  insertMode: CatalogInsertMode;
  canInsertIntoConditionBranch: boolean;
  templates: TemplateOption[];
  onQueryChange: (query: string) => void;
  onCategoryChange: (category: "all" | FlowTemplateCategory) => void;
  onInsertModeChange: (mode: CatalogInsertMode) => void;
  onInsert: (templateId: FlowTemplateId) => void;
  onClose: () => void;
};

export function NodePickerDialog({
  open,
  inputRef,
  query,
  category,
  insertMode,
  templates,
  onQueryChange,
  onCategoryChange,
  onInsert,
  onClose,
}: NodePickerDialogProps) {
  const [focusedIndex, setFocusedIndex] = useState(-1);
  const itemRefs = useRef<(HTMLButtonElement | null)[]>([]);

  // Cap index in case templates list shrunk after filtering
  const safeFocusedIndex = Math.min(focusedIndex, templates.length - 1);

  // Scroll focused item into view
  useEffect(() => {
    if (safeFocusedIndex >= 0) {
      itemRefs.current[safeFocusedIndex]?.scrollIntoView({ block: "nearest" });
    }
  }, [safeFocusedIndex]);

  function handleSearchKeyDown(e: React.KeyboardEvent<HTMLInputElement>) {
    if (e.key === "Escape") {
      e.preventDefault();
      e.stopPropagation();
      onClose();
      return;
    }

    if (e.key === "ArrowDown") {
      e.preventDefault();
      setFocusedIndex((i) => Math.min(i + 1, templates.length - 1));
      return;
    }

    if (e.key === "ArrowUp") {
      e.preventDefault();
      setFocusedIndex((i) => Math.max(i - 1, 0));
      return;
    }

    if (e.key === "Enter") {
      e.preventDefault();
      e.stopPropagation();
      const target = safeFocusedIndex >= 0 ? templates[safeFocusedIndex] : templates[0];
      if (target) onInsert(target.id);
      return;
    }
  }

  if (!open) return null;

  return (
    <>
      {/* Invisible click-away layer — no visual backdrop */}
      <div
        className="absolute inset-0 z-40"
        onClick={onClose}
        aria-hidden
      />

      {/* Slide-in panel from the right */}
      <div className="absolute bottom-0 right-0 top-0 z-50 flex w-80 flex-col border-l border-zinc-200 bg-white shadow-2xl">
        {/* Header */}
        <div className="flex shrink-0 items-center justify-between border-b border-zinc-100 px-4 py-3">
          <div>
            <p className="text-sm font-semibold text-zinc-800">Add an action</p>
            <p className="text-[11px] text-zinc-400">
              Inserting into{" "}
              <span className="font-medium text-zinc-600">{INSERT_MODE_LABELS[insertMode]}</span>
            </p>
          </div>
          <button
            type="button"
            onClick={onClose}
            className="flex size-7 items-center justify-center rounded-md text-zinc-400 transition hover:bg-zinc-100 hover:text-zinc-700"
          >
            <X className="size-4" />
          </button>
        </div>

        {/* Search */}
        <div className="shrink-0 px-3 pt-3">
          <div className="relative">
            <Search className="pointer-events-none absolute left-2.5 top-1/2 size-4 -translate-y-1/2 text-zinc-400" />
            <input
              ref={inputRef}
              type="text"
              value={query}
              onChange={(e) => { onQueryChange(e.target.value); setFocusedIndex(-1); }}
              onKeyDown={handleSearchKeyDown}
              placeholder="Search actions..."
              className="w-full rounded-lg border border-zinc-200 bg-zinc-50 py-2 pl-9 pr-3 text-sm text-zinc-800 placeholder-zinc-400 outline-none transition focus:border-emerald-400 focus:bg-white focus:ring-2 focus:ring-emerald-100"
              autoComplete="off"
            />
          </div>
        </div>

        {/* Category chips */}
        <div className="shrink-0 flex flex-wrap gap-1 px-3 py-2.5">
          {CATEGORY_CHIPS.map((chip) => (
            <button
              key={chip.value}
              type="button"
              onClick={() => onCategoryChange(chip.value)}
              className={`rounded-full px-2.5 py-1 text-[11px] font-medium transition ${
                category === chip.value
                  ? "bg-emerald-100 text-emerald-700 ring-1 ring-emerald-300"
                  : "bg-zinc-100 text-zinc-600 hover:bg-zinc-200"
              }`}
            >
              {chip.label}
            </button>
          ))}
        </div>

        {/* Results */}
        <div className="min-h-0 flex-1 overflow-y-auto">
          {templates.length > 0 ? (
            <div className="px-2 pb-3">
              {templates.map((template, index) => {
                const Icon = TEMPLATE_ICONS[template.id] ?? Database;
                const colorClass = TEMPLATE_COLORS[template.id] ?? "bg-zinc-100 text-zinc-600";
                const isFocused = index === safeFocusedIndex;

                return (
                  <button
                    key={template.id}
                    ref={(el) => {
                      itemRefs.current[index] = el;
                    }}
                    type="button"
                    onClick={() => onInsert(template.id)}
                    onMouseEnter={() => setFocusedIndex(index)}
                    className={`group flex w-full items-center gap-3 rounded-lg px-2.5 py-2.5 text-left transition ${
                      isFocused
                        ? "bg-emerald-50 ring-1 ring-inset ring-emerald-200"
                        : "hover:bg-zinc-50"
                    }`}
                  >
                    <span
                      className={`flex size-8 shrink-0 items-center justify-center rounded-lg ${colorClass}`}
                    >
                      <Icon className="size-4" />
                    </span>
                    <span className="min-w-0 flex-1">
                      <span className="block text-sm font-medium text-zinc-800">
                        {template.label}
                      </span>
                      <span className="block truncate text-[11px] text-zinc-500">
                        {template.description}
                      </span>
                    </span>
                    <ChevronRight
                      className={`size-3.5 shrink-0 transition ${
                        isFocused ? "text-emerald-500" : "text-zinc-300"
                      }`}
                    />
                  </button>
                );
              })}
            </div>
          ) : (
            <div className="flex flex-col items-center justify-center py-12 text-center">
              <Search className="mb-2 size-6 text-zinc-300" />
              <p className="text-sm text-zinc-500">No actions match</p>
              <p className="text-xs text-zinc-400">Try a different search or category</p>
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="shrink-0 border-t border-zinc-100 px-4 py-2">
          <p className="text-[11px] text-zinc-400">
            <kbd className="rounded bg-zinc-100 px-1 py-0.5 font-mono text-[10px]">↑↓</kbd>{" "}
            navigate
            {" · "}
            <kbd className="rounded bg-zinc-100 px-1 py-0.5 font-mono text-[10px]">↵</kbd>{" "}
            select
            {" · "}
            <kbd className="rounded bg-zinc-100 px-1 py-0.5 font-mono text-[10px]">Esc</kbd>{" "}
            close
          </p>
        </div>
      </div>
    </>
  );
}

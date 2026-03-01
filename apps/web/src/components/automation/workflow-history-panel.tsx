"use client";

import { useState } from "react";
import Link from "next/link";
import {
  ArrowLeft,
  CheckCircle,
  ChevronDown,
  ChevronRight,
  Clock,
  Loader,
  XCircle,
  AlertCircle,
} from "lucide-react";

import { Button, StatusBadge, Textarea, buttonVariants } from "@qryvanta/ui";

import type {
  WorkflowResponse,
  WorkflowRunAttemptResponse,
  WorkflowRunResponse,
  WorkflowRunStepTraceResponse,
} from "@/lib/api";
import { formatUtcDateTime } from "@/lib/date-format";
import { cn } from "@/lib/utils";

type WorkflowHistoryPanelProps = {
  workflow: WorkflowResponse;
  runs: WorkflowRunResponse[];
};

function statusTone(status: string): "success" | "critical" | "warning" | "neutral" {
  if (status === "succeeded") return "success";
  if (status === "failed" || status === "dead_lettered") return "critical";
  if (status === "running" || status === "pending" || status === "leased" || status === "queued") return "warning";
  return "neutral";
}

function RunStatusIcon({ status }: { status: string }) {
  if (status === "succeeded") return <CheckCircle className="size-4 text-emerald-500" />;
  if (status === "failed" || status === "dead_lettered") return <XCircle className="size-4 text-red-500" />;
  if (status === "running" || status === "pending" || status === "leased" || status === "queued")
    return <Loader className="size-4 animate-spin text-blue-500" />;
  return <AlertCircle className="size-4 text-amber-500" />;
}

function durationMs(traces: WorkflowRunStepTraceResponse[]): number {
  return traces.reduce((sum, t) => {
    const v = t.duration_ms;
    if (v === null) return sum;
    return sum + (typeof v === "bigint" ? Number(v) : v);
  }, 0);
}

function StepTraceRow({ trace }: { trace: WorkflowRunStepTraceResponse }) {
  const [expanded, setExpanded] = useState(false);
  const tone = trace.status === "failed" ? "text-red-700" : trace.status === "succeeded" ? "text-emerald-700" : "text-zinc-500";

  return (
    <div className="rounded border border-zinc-200 bg-white">
      <button
        type="button"
        className="flex w-full items-center gap-2 px-3 py-2 text-left"
        onClick={() => setExpanded((v) => !v)}
      >
        {expanded ? <ChevronDown className="size-3.5 text-zinc-400" /> : <ChevronRight className="size-3.5 text-zinc-400" />}
        <span className="font-mono text-[11px] text-zinc-600">{trace.step_path}</span>
        <span className={cn("ml-auto text-[11px] font-semibold", tone)}>{trace.status}</span>
        {trace.duration_ms !== null && (
          <span className="text-[11px] text-zinc-400">{typeof trace.duration_ms === "bigint" ? Number(trace.duration_ms) : trace.duration_ms}ms</span>
        )}
      </button>
      {expanded && (
        <div className="space-y-2 border-t border-zinc-100 px-3 py-2">
          {trace.error_message && (
            <p className="rounded bg-red-50 px-2 py-1 text-[11px] text-red-700">{trace.error_message}</p>
          )}
          <div className="grid grid-cols-2 gap-2">
            <div className="space-y-1">
              <p className="text-[10px] font-semibold uppercase tracking-wide text-zinc-400">Input</p>
              <Textarea
                className="font-mono text-[10px]"
                rows={5}
                value={JSON.stringify(trace.input_payload, null, 2)}
                readOnly
              />
            </div>
            <div className="space-y-1">
              <p className="text-[10px] font-semibold uppercase tracking-wide text-zinc-400">Output</p>
              <Textarea
                className="font-mono text-[10px]"
                rows={5}
                value={JSON.stringify(trace.output_payload, null, 2)}
                readOnly
              />
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

function AttemptRow({ attempt }: { attempt: WorkflowRunAttemptResponse }) {
  const [expanded, setExpanded] = useState(false);
  const total = durationMs(attempt.step_traces);

  return (
    <div className="rounded-md border border-zinc-200 bg-zinc-50">
      <button
        type="button"
        className="flex w-full items-center gap-3 px-3 py-2.5 text-left"
        onClick={() => setExpanded((v) => !v)}
      >
        {expanded ? <ChevronDown className="size-3.5 shrink-0 text-zinc-400" /> : <ChevronRight className="size-3.5 shrink-0 text-zinc-400" />}
        <span className="text-xs font-semibold text-zinc-700">Attempt #{attempt.attempt_number}</span>
        <StatusBadge tone={statusTone(attempt.status)}>{attempt.status}</StatusBadge>
        <span className="text-xs text-zinc-500">{formatUtcDateTime(attempt.executed_at)}</span>
        <span className="ml-auto text-xs text-zinc-400">{attempt.step_traces.length} steps · {total}ms</span>
      </button>
      {expanded && (
        <div className="space-y-1.5 border-t border-zinc-200 px-3 py-2">
          {attempt.error_message && (
            <p className="rounded bg-red-50 px-2 py-1 text-[11px] text-red-700">{attempt.error_message}</p>
          )}
          {attempt.step_traces.length > 0 ? (
            attempt.step_traces.map((trace) => (
              <StepTraceRow key={`${trace.step_path}-${attempt.attempt_number}`} trace={trace} />
            ))
          ) : (
            <p className="text-[11px] text-zinc-400">No step traces recorded.</p>
          )}
        </div>
      )}
    </div>
  );
}

function RunRow({ run }: { run: WorkflowRunResponse }) {
  const [expanded, setExpanded] = useState(false);
  const [attempts, setAttempts] = useState<WorkflowRunAttemptResponse[] | null>(null);
  const [loading, setLoading] = useState(false);

  async function loadAttempts() {
    if (attempts !== null) {
      setExpanded((v) => !v);
      return;
    }
    setLoading(true);
    setExpanded(true);
    try {
      const res = await fetch(`/api/workflows/runs/${run.run_id}/attempts`, {
        credentials: "include",
        cache: "no-store",
      });
      if (res.ok) {
        const data = (await res.json()) as WorkflowRunAttemptResponse[];
        setAttempts(data);
      }
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className={cn("border-b border-zinc-100 last:border-0")}>
      <button
        type="button"
        className="flex w-full items-center gap-3 px-4 py-3 text-left transition hover:bg-zinc-50"
        onClick={() => void loadAttempts()}
      >
        <RunStatusIcon status={run.status} />
        <span className="font-mono text-xs text-zinc-500">{run.run_id}</span>
        <StatusBadge tone={statusTone(run.status)}>{run.status}</StatusBadge>
        <span className="text-xs text-zinc-500">{formatUtcDateTime(run.started_at)}</span>
        <span className="ml-auto text-xs text-zinc-400">{run.attempts} attempt{run.attempts !== 1 ? "s" : ""}</span>
        {expanded ? <ChevronDown className="size-3.5 text-zinc-400" /> : <ChevronRight className="size-3.5 text-zinc-400" />}
      </button>
      {expanded && (
        <div className="space-y-2 border-t border-zinc-100 px-4 py-3">
          {loading ? (
            <div className="flex items-center gap-2 text-xs text-zinc-400">
              <Loader className="size-3.5 animate-spin" />
              Loading attempts...
            </div>
          ) : attempts && attempts.length > 0 ? (
            attempts.map((attempt) => (
              <AttemptRow key={`${attempt.run_id}-${attempt.attempt_number}`} attempt={attempt} />
            ))
          ) : (
            <p className="text-xs text-zinc-400">No attempts found.</p>
          )}
        </div>
      )}
    </div>
  );
}

export function WorkflowHistoryPanel({ workflow, runs }: WorkflowHistoryPanelProps) {
  const workflowHrefSafe = encodeURIComponent(workflow.logical_name);

  return (
    <div className="space-y-5">
      {/* Breadcrumb */}
      <div className="flex items-center gap-2">
        <Link
          href="/maker/automation"
          className="flex items-center gap-1 text-sm text-zinc-500 transition hover:text-zinc-800"
        >
          <ArrowLeft className="size-3.5" />
          All Flows
        </Link>
        <span className="text-zinc-300">/</span>
        <span className="text-sm font-medium text-zinc-700">{workflow.display_name}</span>
        <span className="text-zinc-300">/</span>
        <span className="text-sm text-zinc-500">Run History</span>
      </div>

      {/* Workflow header */}
      <div className="flex items-start justify-between gap-4 rounded-lg border border-zinc-200 bg-white p-4">
        <div className="space-y-1">
          <div className="flex items-center gap-2">
            <h1 className="text-lg font-semibold text-zinc-900">{workflow.display_name}</h1>
            <span className={cn(
              "rounded-full px-2 py-0.5 text-[11px] font-semibold",
              workflow.is_enabled
                ? "bg-emerald-100 text-emerald-700"
                : "bg-zinc-100 text-zinc-500",
            )}>
              {workflow.is_enabled ? "Enabled" : "Disabled"}
            </span>
          </div>
          <p className="font-mono text-xs text-zinc-400">{workflow.logical_name}</p>
          <div className="flex items-center gap-3 pt-1">
            <span className="flex items-center gap-1 text-xs text-zinc-500">
              <Clock className="size-3.5" />
              Trigger: {workflow.trigger_type}
            </span>
            <span className="text-xs text-zinc-500">
              Max attempts: {workflow.max_attempts}
            </span>
            {workflow.description && (
              <span className="text-xs text-zinc-500">{workflow.description}</span>
            )}
          </div>
        </div>
        <div className="flex shrink-0 gap-2">
          <Link
            href={`/maker/automation/${workflowHrefSafe}/edit`}
            className={cn(buttonVariants({ size: "sm", variant: "outline" }))}
          >
            Edit Workflow
          </Link>
        </div>
      </div>

      {/* Run count summary */}
      <div className="flex items-center gap-3">
        <p className="text-sm font-medium text-zinc-700">
          {runs.length} run{runs.length !== 1 ? "s" : ""}
        </p>
        {runs.filter(r => r.status === "failed" || r.status === "dead_lettered").length > 0 && (
          <span className="text-sm font-semibold text-red-600">
            {runs.filter(r => r.status === "failed" || r.status === "dead_lettered").length} failed
          </span>
        )}
        {runs.filter(r => r.status === "running" || r.status === "pending").length > 0 && (
          <span className="text-sm font-semibold text-blue-600">
            {runs.filter(r => r.status === "running" || r.status === "pending").length} running
          </span>
        )}
      </div>

      {/* Runs list */}
      {runs.length > 0 ? (
        <div className="overflow-hidden rounded-lg border border-zinc-200 bg-white">
          {/* Column header */}
          <div className="grid grid-cols-[auto_1fr_auto_auto_auto_auto] items-center gap-x-3 border-b border-zinc-100 bg-zinc-50 px-4 py-2">
            <div className="size-4" />
            <p className="text-[11px] font-semibold uppercase tracking-wide text-zinc-500">Run ID</p>
            <p className="text-[11px] font-semibold uppercase tracking-wide text-zinc-500">Status</p>
            <p className="text-[11px] font-semibold uppercase tracking-wide text-zinc-500">Started</p>
            <p className="text-[11px] font-semibold uppercase tracking-wide text-zinc-500">Attempts</p>
            <div className="size-4" />
          </div>
          {runs.map((run) => (
            <RunRow key={run.run_id} run={run} />
          ))}
        </div>
      ) : (
        <div className="rounded-lg border border-zinc-200 bg-white px-6 py-10 text-center">
          <p className="text-sm font-medium text-zinc-600">No runs yet</p>
          <p className="mt-1 text-xs text-zinc-400">
            Trigger or test run this workflow to see history here.
          </p>
          <div className="mt-4">
            <Link
              href={`/maker/automation/${workflowHrefSafe}/edit`}
              className={cn(buttonVariants({ size: "sm" }))}
            >
              Open Editor
            </Link>
          </div>
        </div>
      )}
    </div>
  );
}

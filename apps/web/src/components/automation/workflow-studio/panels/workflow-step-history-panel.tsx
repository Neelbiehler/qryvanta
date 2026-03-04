"use client";

import { useMemo } from "react";
import { CheckCircle2, Clock3, Loader, XCircle } from "lucide-react";

import { Select, StatusBadge } from "@qryvanta/ui";

import type {
  WorkflowRunAttemptResponse,
  WorkflowRunResponse,
  WorkflowRunStepTraceResponse,
} from "@/lib/api";
import { formatUtcDateTime } from "@/lib/date-format";
import { cn } from "@/lib/utils";

type WorkflowStepHistoryPanelProps = {
  open: boolean;
  workflowLogicalName: string;
  workflowDisplayName: string;
  runs: WorkflowRunResponse[];
  selectedRunId: string | null;
  attempts: WorkflowRunAttemptResponse[];
  selectedAttemptNumber: number | null;
  activeAttempt: WorkflowRunAttemptResponse | null;
  loadingAttemptsRunId: string | null;
  onSelectRun: (runId: string) => void;
  onSelectAttempt: (attemptNumber: number | null) => void;
  onFocusStepPath: (stepPath: string) => void;
};

function runStatusTone(
  status: string,
): "success" | "critical" | "warning" | "neutral" {
  if (status === "succeeded") return "success";
  if (status === "failed" || status === "dead_lettered") return "critical";
  if (
    status === "running" ||
    status === "pending" ||
    status === "leased" ||
    status === "queued"
  ) {
    return "warning";
  }

  return "neutral";
}

function traceStatusClassName(status: string): string {
  if (status === "succeeded") return "text-emerald-700";
  if (status === "failed") return "text-red-700";
  return "text-zinc-500";
}

function traceStatusIcon(status: string) {
  if (status === "succeeded") return <CheckCircle2 className="size-3.5 text-emerald-600" />;
  if (status === "failed") return <XCircle className="size-3.5 text-red-600" />;
  if (
    status === "running" ||
    status === "pending" ||
    status === "leased" ||
    status === "queued"
  ) {
    return <Loader className="size-3.5 animate-spin text-blue-600" />;
  }
  return <Clock3 className="size-3.5 text-zinc-500" />;
}

function durationText(trace: WorkflowRunStepTraceResponse): string {
  if (trace.duration_ms === null) {
    return "-";
  }

  return `${typeof trace.duration_ms === "bigint" ? Number(trace.duration_ms) : trace.duration_ms}ms`;
}

export function WorkflowStepHistoryPanel({
  open,
  workflowLogicalName,
  workflowDisplayName,
  runs,
  selectedRunId,
  attempts,
  selectedAttemptNumber,
  activeAttempt,
  loadingAttemptsRunId,
  onSelectRun,
  onSelectAttempt,
  onFocusStepPath,
}: WorkflowStepHistoryPanelProps) {
  const sortedRuns = useMemo(() => {
    return [...runs].sort((left, right) => {
      const leftTime = Date.parse(left.started_at);
      const rightTime = Date.parse(right.started_at);
      return (Number.isFinite(rightTime) ? rightTime : 0) - (Number.isFinite(leftTime) ? leftTime : 0);
    });
  }, [runs]);

  const sortedAttempts = useMemo(() => {
    return [...attempts].sort((left, right) => right.attempt_number - left.attempt_number);
  }, [attempts]);

  const activeRun =
    selectedRunId === null
      ? null
      : sortedRuns.find((run) => run.run_id === selectedRunId) ?? null;

  if (!open) {
    return null;
  }

  return (
    <div className="absolute bottom-3 left-3 top-[52px] z-30 flex w-[340px] flex-col overflow-hidden rounded-xl border border-zinc-200 bg-white shadow-lg">
      <div className="space-y-2 border-b border-zinc-200 px-3 py-2.5">
        <p className="text-[10px] font-semibold uppercase tracking-[0.12em] text-zinc-500">
          Step History
        </p>
        <div className="space-y-1">
          <p className="truncate text-sm font-semibold text-zinc-900">{workflowDisplayName}</p>
          <p className="truncate font-mono text-[11px] text-zinc-500">{workflowLogicalName}</p>
        </div>
      </div>

      <div className="min-h-0 flex-1 space-y-3 overflow-y-auto p-3">
        {sortedRuns.length === 0 ? (
          <p className="rounded-md border border-zinc-200 bg-zinc-50 px-2 py-2 text-xs text-zinc-500">
            No runs available for this workflow yet.
          </p>
        ) : (
          <>
            <div className="space-y-1.5">
              <p className="text-[11px] font-medium text-zinc-600">Run</p>
              <Select
                value={selectedRunId ?? ""}
                onChange={(event) => onSelectRun(event.target.value)}
              >
                <option value="">Select a run</option>
                {sortedRuns.map((run) => (
                  <option key={run.run_id} value={run.run_id}>
                    {run.run_id} · {run.status}
                  </option>
                ))}
              </Select>
              {activeRun ? (
                <div className="flex items-center gap-2 text-[11px] text-zinc-500">
                  <StatusBadge tone={runStatusTone(activeRun.status)}>
                    {activeRun.status}
                  </StatusBadge>
                  <span>{formatUtcDateTime(activeRun.started_at)}</span>
                </div>
              ) : null}
            </div>

            <div className="space-y-1.5">
              <p className="text-[11px] font-medium text-zinc-600">Attempt</p>
              <Select
                value={selectedAttemptNumber === null ? "" : String(selectedAttemptNumber)}
                onChange={(event) => {
                  const value = event.target.value.trim();
                  if (value.length === 0) {
                    onSelectAttempt(null);
                    return;
                  }

                  onSelectAttempt(Number.parseInt(value, 10));
                }}
                disabled={selectedRunId === null}
              >
                <option value="">Latest attempt</option>
                {sortedAttempts.map((attempt) => (
                  <option key={attempt.attempt_number} value={String(attempt.attempt_number)}>
                    Attempt #{attempt.attempt_number} · {attempt.status}
                  </option>
                ))}
              </Select>
              {loadingAttemptsRunId === selectedRunId ? (
                <div className="flex items-center gap-1.5 text-[11px] text-zinc-400">
                  <Loader className="size-3.5 animate-spin" />
                  Loading attempts...
                </div>
              ) : null}
            </div>

            <div className="space-y-1.5">
              <p className="text-[11px] font-medium text-zinc-600">
                Traced Steps
                {activeAttempt ? ` (${activeAttempt.step_traces.length})` : ""}
              </p>
              {activeAttempt?.error_message ? (
                <p className="rounded border border-red-200 bg-red-50 px-2 py-1.5 text-[11px] text-red-700">
                  {activeAttempt.error_message}
                </p>
              ) : null}
              {activeAttempt && activeAttempt.step_traces.length > 0 ? (
                <div className="space-y-1">
                  {activeAttempt.step_traces.map((trace) => (
                    <button
                      key={trace.step_path}
                      type="button"
                      className="flex w-full items-center gap-2 rounded-md border border-zinc-200 bg-white px-2 py-1.5 text-left hover:border-emerald-300 hover:bg-emerald-50"
                      onClick={() => onFocusStepPath(trace.step_path)}
                    >
                      {traceStatusIcon(trace.status)}
                      <div className="min-w-0 flex-1">
                        <p className="truncate font-mono text-[11px] text-zinc-700">{trace.step_path}</p>
                        <p className={cn("text-[10px] font-semibold", traceStatusClassName(trace.status))}>
                          {trace.status}
                        </p>
                      </div>
                      <span className="text-[10px] text-zinc-400">{durationText(trace)}</span>
                    </button>
                  ))}
                </div>
              ) : (
                <p className="rounded-md border border-zinc-200 bg-zinc-50 px-2 py-2 text-[11px] text-zinc-500">
                  No step traces for this attempt.
                </p>
              )}
            </div>
          </>
        )}
      </div>
    </div>
  );
}
